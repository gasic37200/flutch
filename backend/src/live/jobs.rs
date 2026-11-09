//! 주기 작업(architecture §5): 60초 유예·3시간 자동 종료, 재시작 시 LiveKit 동기화.

use std::collections::HashSet;

use super::{db, is_viewer, lock};
use crate::AppState;
use crate::broadcast::{EndReason, end_live, room_name};
use crate::error::AppError;
use crate::realtime::{Event, WsEvent};

/// 자동 종료 몇 초 전에 경고하나(features/03 "3시간 10분 전 경고").
const WARN_BEFORE_SECS: i64 = 10 * 60;

/// 유예가 지난 LIVE → 종료, 최대 길이 지난 LIVE → 종료, 10분 전 → 소유자에게 한 번 경고.
/// 방송 하나가 실패해도 나머지는 계속한다(로그만).
pub async fn check_limits(state: &AppState) -> Result<(), AppError> {
    let cfg = &state.config.live;
    let rows = db::deadlines(
        &state.db,
        cfg.grace_secs,
        cfg.max_duration_secs,
        WARN_BEFORE_SECS,
    )
    .await?;
    lock(&state.live.warned).retain(|id| rows.iter().any(|r| r.id == *id));
    for r in rows {
        let res = if r.over_max {
            end_live(state, r.id, EndReason::MaxDuration).await
        } else if r.grace_over {
            expire_grace(state, r.id, r.publisher_identity.as_deref()).await
        } else {
            if r.warn && lock(&state.live.warned).insert(r.id) {
                let e = WsEvent::BroadcastDurationWarning { ends_at: r.ends_at };
                state.realtime.publish(r.id, Event::owner_only(e));
            }
            Ok(())
        };
        if let Err(e) = res {
            tracing::error!(broadcast_id = r.id, error = %e, "live limit check failed");
        }
    }
    Ok(())
}

/// 유예 만료. 웹훅 순서가 뒤바뀌면(같은 신원 재접속 시 새 참가가 옛 퇴장보다 먼저 옴) 송출자가 실제로는
/// 접속해 있을 수 있어 LiveKit에 한 번 확인한다. 확인 실패면 규칙대로 종료.
async fn expire_grace(state: &AppState, id: i64, publisher: Option<&str>) -> Result<(), AppError> {
    match state.livekit.list_participants(&room_name(id)).await {
        Ok(list) => {
            if let Some(p) = publisher.filter(|p| list.iter().any(|i| i == p)) {
                if db::end_grace(&state.db, id, p).await? {
                    tracing::info!(broadcast_id = id, "publisher still in room, grace cleared");
                    state
                        .realtime
                        .publish(id, Event::public(WsEvent::BroadcastResumed {}));
                }
                return Ok(());
            }
        }
        Err(e) => {
            tracing::error!(broadcast_id = id, error = %e, "livekit check before grace end failed, ending")
        }
    }
    end_live(state, id, EndReason::Disconnect).await
}

/// 재시작 동기화: LIVE 방송마다 LiveKit 실제 참가자로 시청자 집합을 복구하고,
/// 송출자가 없으면 유예 시작(→ 60초 뒤 자연 종료), 유예 중인데 송출자가 있으면 해제.
/// LiveKit을 전부 읽은 뒤에만 DB를 바꾼다(호출 실패 시 DB는 그대로).
pub async fn sync(state: &AppState) -> Result<(), AppError> {
    let live = db::all_live(&state.db).await?;
    let mut rooms = Vec::with_capacity(live.len());
    for b in live {
        let list = state
            .livekit
            .list_participants(&room_name(b.id))
            .await
            .map_err(|e| AppError::Internal(format!("livekit list participants: {e}")))?;
        rooms.push((b, list));
    }
    let n = rooms.len();
    for (b, list) in rooms {
        let viewers: HashSet<String> = list
            .iter()
            .filter(|i| is_viewer(i, b.owner_id))
            .cloned()
            .collect();
        if let Some(count) = state.live.update_viewers(b.id, |s| *s = viewers) {
            db::raise_peak(&state.db, b.id, count).await?;
            state
                .realtime
                .publish(b.id, Event::public(WsEvent::ViewerCount { count }));
        }
        match b.publisher_identity.filter(|p| list.contains(p)) {
            Some(p) if b.away => {
                if db::end_grace(&state.db, b.id, &p).await? {
                    state
                        .realtime
                        .publish(b.id, Event::public(WsEvent::BroadcastResumed {}));
                }
            }
            Some(_) => {}
            None => {
                if let Some(since) = db::start_grace(&state.db, b.id, None).await? {
                    tracing::info!(
                        broadcast_id = b.id,
                        "publisher absent after restart, grace started"
                    );
                    state.realtime.publish(
                        b.id,
                        Event::public(WsEvent::BroadcastReconnecting { since }),
                    );
                }
            }
        }
    }
    tracing::info!(broadcasts = n, "live state synced with livekit");
    Ok(())
}
