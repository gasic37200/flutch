use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::{Extension, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{
    BurstRequest, BurstResult, ClutchError, ClutchResult, Limits, Source, add_points, db, publish,
};
use crate::AppState;
use crate::broadcast::{Broadcast, Page};
use crate::error::AppError;
use crate::extract::{CurrentUser, JsonBody};
use crate::wallet::{self, DebitKind, DonationError, DonationKind, NewDonation};

pub async fn create(
    State(state): State<Arc<AppState>>,
    Extension(limits): Extension<Arc<Limits>>,
    user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<Json<ClutchResult>, AppError> {
    limits.hit(user.id, id)?;
    let cfg = &state.config.clutch;
    let mut tx = state.db.begin().await?;
    // 같은 사용자의 동시 요청이 무료 횟수를 함께 넘기지 않게 직렬화(잠금 순서: 사용자 → 방송 → 지갑 → 게이지).
    db::lock(&mut tx, &format!("clutch:user:{}:{id}", user.id)).await?;
    let (owner_id, status) = db::broadcast_for_share(&mut tx, id)
        .await?
        .ok_or(AppError::NotFound)?;
    if owner_id == user.id {
        return Err(ClutchError::SelfClutch.into());
    }
    if status != "LIVE" {
        return Err(ClutchError::NotLive.into());
    }
    let used = db::free_used(&mut *tx, user.id, id, cfg.free_window_secs).await?;
    let free = used < cfg.free_per_window;
    let mut balance = None;
    if !free {
        // 영구 밴은 유료만 막는다(무료는 허용). 임시차단은 둘 다 허용.
        if crate::chat::ban_state(&state.db, owner_id, user.id)
            .await?
            .is_some_and(|b| b.permanent)
        {
            return Err(ClutchError::Banned.into());
        }
        let posted =
            wallet::debit(&mut tx, user.id, 1, DebitKind::Clutch { broadcast_id: id }).await?;
        wallet::insert_donation(
            &mut tx,
            &NewDonation {
                broadcast_id: id,
                donor_id: user.id,
                kind: DonationKind::Clutch,
                amount: 1,
                anonymous: true,
                message: None,
                ledger_entry_id: posted.ledger_entry_id,
            },
        )
        .await?;
        balance = Some(posted.balance_after);
    }
    let source = if free { Source::Free } else { Source::Paid };
    let out = add_points(&mut tx, cfg, id, user.id, source, cfg.clutch_points).await?;
    tx.commit().await?;
    publish(&state, id, &out);
    Ok(Json(ClutchResult {
        paid: !free,
        combo: if free { used as i32 + 1 } else { 0 },
        free_clutches_left: (cfg.free_per_window - used - 1).max(0) as i32,
        balance,
        gauge: out.gauge,
    }))
}

/// SUPER BURST: 가격 차감 + 후원 기록(kind burst) + 게이지 점수를 한 트랜잭션으로.
/// 잠금 순서: 방송 FOR SHARE → 지갑 차감 → 게이지(후원·클러치와 같은 순서라 교착 없음).
pub async fn burst(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path(id): Path<i64>,
    JsonBody(req): JsonBody<BurstRequest>,
) -> Result<Json<BurstResult>, AppError> {
    let cfg = &state.config.clutch;
    let price = cfg.burst_price;
    let mut tx = state.db.begin().await?;
    // FOR SHARE: 차감·기록이 커밋되기 전에 방송이 종료되지 않는다(LIVE 중 버스트만 남는다).
    let (owner_id, status) = db::broadcast_for_share(&mut tx, id)
        .await?
        .ok_or(AppError::NotFound)?;
    if owner_id == user.id {
        return Err(ClutchError::SelfClutch.into());
    }
    if crate::chat::ban_state(&state.db, owner_id, user.id)
        .await?
        .is_some_and(|b| b.permanent)
    {
        return Err(DonationError::Banned.into());
    }
    if status != "LIVE" {
        return Err(ClutchError::NotLive.into());
    }
    let posted = wallet::debit(
        &mut tx,
        user.id,
        price,
        DebitKind::Burst { broadcast_id: id },
    )
    .await?;
    let donation = wallet::insert_donation(
        &mut tx,
        &NewDonation {
            broadcast_id: id,
            donor_id: user.id,
            kind: DonationKind::Burst,
            amount: price,
            anonymous: req.anonymous,
            message: None,
            ledger_entry_id: posted.ledger_entry_id,
        },
    )
    .await?;
    let out = add_points(
        &mut tx,
        cfg,
        id,
        user.id,
        Source::Burst,
        price * cfg.points_per_bit,
    )
    .await?;
    tx.commit().await?;

    metrics::counter!("donations_total").increment(1);
    wallet::publish_donation(&state, donation.clone()).await;
    publish(&state, id, &out);
    Ok(Json(BurstResult {
        donation,
        balance: posted.balance_after,
        gauge: out.gauge,
    }))
}

/// 가상 클립 구간: SUPER CLUTCH 발동 30초 전부터 45초(발동 15초 후까지). 파일은 만들지 않는다.
const CLIP_BEFORE_SECS: i32 = 30;
const CLIP_LEN_SECS: i32 = 45;

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Clip {
    /// super_clutch id
    pub id: i64,
    pub recording_id: i64,
    pub video_url: String,
    pub broadcast: Broadcast,
    pub start_sec: i32,
    pub duration_sec: i32,
    pub triggered_at: DateTime<Utc>,
}

#[derive(Deserialize, Default)]
pub struct ClipsQuery {
    cursor: Option<String>,
    limit: Option<String>,
}

pub async fn clips(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ClipsQuery>,
) -> Result<Json<Page<Clip>>, AppError> {
    let num = |field: &str, v: &Option<String>| -> Result<Option<i64>, AppError> {
        v.as_deref()
            .map(|s| {
                s.parse().map_err(|_| {
                    AppError::Validation(
                        [(field.to_owned(), "숫자로 입력해 주세요.".to_owned())].into(),
                    )
                })
            })
            .transpose()
    };
    let limit = num("limit", &q.limit)?.unwrap_or(20).clamp(1, 50);
    let mut rows = db::clips(
        &state.db,
        num("cursor", &q.cursor)?,
        CLIP_BEFORE_SECS,
        CLIP_LEN_SECS,
        limit + 1,
    )
    .await?;
    let next_cursor = (rows.len() as i64 > limit).then(|| {
        rows.truncate(limit as usize);
        rows.last().map(|r| r.id.to_string())
    });
    let base = &state.config.recording.public_base_url;
    let mut items = Vec::with_capacity(rows.len());
    for r in rows {
        // shortcut: 클립마다 방송을 개별 조회(최대 51회), 목록 50개 한도라 허용 — 부하가 보이면 한 쿼리로 묶는다.
        // 조회 사이에 숨겨진 방송은 건너뛴다
        let Some(broadcast) = crate::broadcast::find_one(&state, r.broadcast_id).await? else {
            continue;
        };
        items.push(Clip {
            id: r.id,
            recording_id: r.recording_id,
            video_url: format!("{base}/{}", r.object_key),
            broadcast,
            start_sec: r.start_sec,
            duration_sec: r.duration_sec,
            triggered_at: r.triggered_at,
        });
    }
    Ok(Json(Page {
        items,
        next_cursor: next_cursor.flatten(),
    }))
}
