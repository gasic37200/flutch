use std::sync::Arc;

use axum::extract::{Path, State};
use axum::{Extension, Json};

use super::{
    BurstRequest, BurstResult, ClutchError, ClutchResult, Limits, Source, add_points, db, publish,
};
use crate::AppState;
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
