//! 잔액 후원·후원 목록·알림 스킵·받은 후원 통계(api.md §6). 익명 후원자의 실명은
//! `Donation::visible_to`로만 가린다 — 공개 응답·공개 이벤트가 모두 그 함수를 지난다.

use std::collections::BTreeMap;
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use chrono::{DateTime, Days, NaiveTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::db;
use super::{
    BroadcastRef, DebitKind, Donation, DonationError, DonationInput, DonationKind, NewDonation,
    check_donation_input, debit, donation_target, insert_donation, publish_donation,
};
use crate::AppState;
use crate::broadcast::BroadcastError;
use crate::error::AppError;
use crate::extract::{CurrentUser, JsonBody, OptionalUser};
use crate::realtime::{Event, WsEvent};

const DEFAULT_LIMIT: i64 = 20;
const MAX_LIMIT: i64 = 50;

/// POST /broadcasts/{id}/donations 응답
#[derive(Serialize, TS)]
#[ts(export)]
pub struct DonateResponse {
    /// 내 후원이라 실명
    pub donation: Donation,
    pub balance: i64,
}

/// GET /broadcasts/{id}/donations 응답 = `Page<Donation>` + 합계
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DonationList {
    pub items: Vec<Donation>,
    pub next_cursor: Option<String>,
    /// 이 방송이 받은 비트 합계(donation + clutch + burst)
    pub total: i64,
    /// donation·burst 건수
    pub count: i64,
    /// 유료 클러치 합계(목록에는 건별로 나오지 않음)
    pub clutch_total: i64,
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StatsBroadcast {
    #[serde(flatten)]
    #[ts(flatten)]
    pub broadcast: BroadcastRef,
    pub started_at: DateTime<Utc>,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct DonationStatsItem {
    pub broadcast: StatsBroadcast,
    pub total: i64,
    /// donation·burst 건수
    pub count: i64,
}

/// GET /me/donation-stats 응답
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DonationStats {
    pub items: Vec<DonationStatsItem>,
    pub next_cursor: Option<String>,
    /// 누적 받은 비트
    pub total_received: i64,
    /// 최근 7일(KST, 오늘 포함 달력 7일) 합계
    pub last7_days: i64,
}

#[derive(Deserialize)]
pub struct PageQuery {
    cursor: Option<String>,
    limit: Option<String>,
}

fn field_error(field: &str, reason: &str) -> AppError {
    AppError::Validation(BTreeMap::from([(field.to_owned(), reason.to_owned())]))
}

/// `(before id, limit)`. 커서는 이전 페이지 마지막 항목의 id.
fn page(q: PageQuery) -> Result<(Option<i64>, i64), AppError> {
    let before = q
        .cursor
        .map(|c| {
            c.parse::<i64>()
                .map_err(|_| field_error("cursor", "올바르지 않은 커서예요."))
        })
        .transpose()?;
    let limit = match q.limit.map(|l| l.parse::<i64>()) {
        None => DEFAULT_LIMIT,
        Some(Ok(n)) => n.clamp(1, MAX_LIMIT),
        Some(Err(_)) => return Err(field_error("limit", "숫자로 입력해 주세요.")),
    };
    Ok((before, limit))
}

/// `limit + 1`개를 읽은 행을 자르고 다음 커서(마지막 id)를 만든다.
fn next_cursor<T>(rows: &mut Vec<T>, limit: i64, id: impl Fn(&T) -> i64) -> Option<String> {
    if rows.len() as i64 > limit {
        rows.truncate(limit as usize);
        rows.last().map(|r| id(r).to_string())
    } else {
        None
    }
}

/// 잔액 후원. 검사 순서 404 → 403(소유자·영구 밴) → 422 → 409(LIVE 아님·잔액 부족).
pub async fn donate(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path(broadcast_id): Path<i64>,
    JsonBody(req): JsonBody<DonationInput>,
) -> Result<(StatusCode, Json<DonateResponse>), AppError> {
    donation_target(&state.db, user.id, broadcast_id).await?;
    let message = check_donation_input(req.amount, req.message.as_deref())?;

    let mut tx = state.db.begin().await?;
    // FOR SHARE: 차감·기록이 커밋되기 전에 방송이 종료되지 않는다(LIVE 중 후원만 남는다).
    match db::broadcast_status_for_share(&mut tx, broadcast_id).await? {
        Some((_, status)) if status == "LIVE" => {}
        _ => return Err(DonationError::NotLive.into()),
    }
    let posted = debit(
        &mut tx,
        user.id,
        req.amount,
        DebitKind::Donation { broadcast_id },
    )
    .await?;
    let donation = insert_donation(
        &mut tx,
        &NewDonation {
            broadcast_id,
            donor_id: user.id,
            kind: DonationKind::Donation,
            amount: req.amount,
            anonymous: req.anonymous,
            message: message.as_deref(),
            ledger_entry_id: posted.ledger_entry_id,
        },
    )
    .await?;
    tx.commit().await?;

    metrics::counter!("donations_total").increment(1);
    publish_donation(&state, donation.clone()).await;
    Ok((
        StatusCode::CREATED,
        Json(DonateResponse {
            donation,
            balance: posted.balance_after,
        }),
    ))
}

/// 후원 목록 + 합계. 소유자 요청이면 익명 후원도 실명, 그 외(비로그인 포함)는 `donor=null`.
pub async fn list(
    State(state): State<Arc<AppState>>,
    OptionalUser(user): OptionalUser,
    Path(broadcast_id): Path<i64>,
    Query(q): Query<PageQuery>,
) -> Result<Json<DonationList>, AppError> {
    let (owner_id, _) = db::broadcast_info(&state.db, broadcast_id)
        .await?
        .ok_or(AppError::NotFound)?;
    let (before, limit) = page(q)?;
    let is_owner = user.is_some_and(|u| u.id == owner_id);

    let mut rows = db::list_donations(&state.db, broadcast_id, before, limit + 1).await?;
    let next_cursor = next_cursor(&mut rows, limit, |r| r.id);
    let items = rows
        .into_iter()
        .map(|r| Ok(r.into_donation()?.visible_to(is_owner)))
        .collect::<Result<_, AppError>>()?;
    let t = db::donation_totals(&state.db, broadcast_id).await?;
    Ok(Json(DonationList {
        items,
        next_cursor,
        total: t.total,
        count: t.count,
        clutch_total: t.clutch_total,
    }))
}

/// 표시 중인 알림 넘기기. 서버는 이벤트만 보내고, 알림 큐(10초·순서)는 클라이언트가 관리한다.
pub async fn skip(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let (broadcast_id, owner_id) = db::donation_broadcast(&state.db, id)
        .await?
        .ok_or(AppError::NotFound)?;
    if owner_id != user.id {
        return Err(BroadcastError::NotOwner.into());
    }
    state
        .realtime
        .publish(broadcast_id, Event::public(WsEvent::DonationSkipped { id }));
    Ok(StatusCode::NO_CONTENT)
}

/// KST 달력으로 오늘 포함 최근 7일의 시작(6일 전 00:00 KST). KST는 서머타임이 없어 고정 +9시간.
fn kst_7_days_start(now: DateTime<Utc>) -> DateTime<Utc> {
    let nine = TimeDelta::hours(9);
    let today = (now + nine).date_naive();
    (today - Days::new(6)).and_time(NaiveTime::MIN).and_utc() - nine
}

pub async fn stats(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Query(q): Query<PageQuery>,
) -> Result<Json<DonationStats>, AppError> {
    let (before, limit) = page(q)?;
    let mut rows = db::stats(&state.db, user.id, before, limit + 1).await?;
    let next_cursor = next_cursor(&mut rows, limit, |r| r.id);
    let (total_received, last7_days) =
        db::received(&state.db, user.id, kst_7_days_start(Utc::now())).await?;
    Ok(Json(DonationStats {
        items: rows
            .into_iter()
            .map(|r| DonationStatsItem {
                broadcast: StatsBroadcast {
                    broadcast: BroadcastRef {
                        id: r.id,
                        title: r.title,
                        hidden: r.hidden,
                    },
                    started_at: r.started_at,
                },
                total: r.total,
                count: r.count,
            })
            .collect(),
        next_cursor,
        total_received,
        last7_days,
    }))
}

#[cfg(test)]
mod tests {
    use super::kst_7_days_start;

    #[test]
    fn kst_7_days_bounds() {
        for (now, start) in [
            // KST 11/10 08:59 → 11/4 00:00 KST
            ("2026-11-09T23:59:00Z", "2026-11-03T15:00:00+00:00"),
            // KST 11/10 09:00(UTC 자정) — 같은 KST 날짜
            ("2026-11-10T00:00:00Z", "2026-11-03T15:00:00+00:00"),
            // KST 11/11 00:00 → 하루 밀림
            ("2026-11-10T15:00:00Z", "2026-11-04T15:00:00+00:00"),
        ] {
            let now = now.parse().unwrap();
            assert_eq!(kst_7_days_start(now).to_rfc3339(), start, "{now}");
        }
    }
}
