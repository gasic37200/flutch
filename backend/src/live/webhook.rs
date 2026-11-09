//! `POST /livekit/webhook` (api.md §10). LiveKit이 참가·퇴장·방 종료를 알린다.

use std::collections::BTreeMap;
use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, header};
use serde::Deserialize;

use super::{LiveError, db, is_viewer};
use crate::AppState;
use crate::broadcast::room_name;
use crate::error::AppError;
use crate::livekit;
use crate::realtime::{Event, WsEvent};

/// LiveKit `WebhookEvent`(protojson, camelCase)에서 쓰는 필드만.
#[derive(Deserialize)]
struct WebhookEvent {
    event: String,
    room: Option<Room>,
    participant: Option<Participant>,
}

#[derive(Deserialize)]
struct Room {
    name: String,
}

#[derive(Deserialize)]
struct Participant {
    identity: String,
    #[serde(default)]
    permission: Permission,
}

/// protojson은 false를 생략한다 → 없으면 false.
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct Permission {
    #[serde(default)]
    can_publish: bool,
}

pub async fn receive(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, AppError> {
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .map(|t| t.trim_start_matches("Bearer "))
        .unwrap_or_default();
    let now = chrono::Utc::now().timestamp();
    if !livekit::verify_webhook(&state.config.livekit, token, &body, now) {
        tracing::warn!("livekit webhook signature invalid");
        return Err(LiveError::WebhookSignature.into());
    }
    let ev: WebhookEvent = serde_json::from_slice(&body)
        .map_err(|e| AppError::Validation(BTreeMap::from([("body".to_owned(), e.to_string())])))?;
    // 우리 방(`broadcast-<id>`)이 아니면 무시
    let Some(id) = ev
        .room
        .as_ref()
        .and_then(|r| r.name.strip_prefix("broadcast-"))
        .and_then(|n| n.parse::<i64>().ok())
    else {
        return Ok(StatusCode::OK);
    };
    tracing::debug!(broadcast_id = id, event = %ev.event, "livekit webhook");
    match (ev.event.as_str(), ev.participant) {
        ("participant_joined", Some(p)) => joined(&state, id, p).await?,
        ("participant_left", Some(p)) => left(&state, id, &p.identity).await?,
        ("room_finished", _) => room_finished(&state, id).await?,
        // shortcut: egress_ended(녹화 ready/failed)는 T25가 처리한다. 지금은 다른 이벤트와 같이 무시.
        _ => {}
    }
    Ok(StatusCode::OK)
}

async fn joined(state: &AppState, id: i64, p: Participant) -> Result<(), AppError> {
    let Some(b) = db::live(&state.db, id).await? else {
        return Ok(());
    };
    if b.publisher_identity.as_deref() == Some(p.identity.as_str()) {
        if db::end_grace(&state.db, id, &p.identity).await? {
            tracing::info!(broadcast_id = id, identity = %p.identity, "publisher resumed");
            state
                .realtime
                .publish(id, Event::public(WsEvent::BroadcastResumed {}));
        }
        return Ok(());
    }
    if p.permission.can_publish {
        // 기기 전환으로 밀려난 기기가 옛 송출 토큰(TTL 4시간)으로 다시 들어왔다 → 내보낸다
        tracing::info!(broadcast_id = id, identity = %p.identity, "stale publisher rejoined, removing");
        if let Err(e) = state
            .livekit
            .remove_participant(&room_name(id), &p.identity)
            .await
        {
            tracing::error!(broadcast_id = id, identity = %p.identity, error = %e, "livekit remove stale publisher failed");
        }
        return Ok(());
    }
    if is_viewer(&p.identity, b.owner_id)
        && let Some(count) = state.live.update_viewers(id, |s| {
            s.insert(p.identity);
        })
    {
        db::raise_peak(&state.db, id, count).await?;
        state
            .realtime
            .publish(id, Event::public(WsEvent::ViewerCount { count }));
    }
    Ok(())
}

async fn left(state: &AppState, id: i64, identity: &str) -> Result<(), AppError> {
    let count = state.live.update_viewers(id, |s| {
        s.remove(identity);
    });
    let Some(b) = db::live(&state.db, id).await? else {
        return Ok(());
    };
    if let Some(count) = count {
        state
            .realtime
            .publish(id, Event::public(WsEvent::ViewerCount { count }));
    }
    if b.publisher_identity.as_deref() == Some(identity)
        && let Some(since) = db::start_grace(&state.db, id, Some(identity)).await?
    {
        tracing::info!(
            broadcast_id = id,
            identity,
            "publisher disconnected, grace started"
        );
        state
            .realtime
            .publish(id, Event::public(WsEvent::BroadcastReconnecting { since }));
    }
    Ok(())
}

/// LiveKit은 마지막 참가자가 나가고 잠시(departure timeout, 기본 20초) 뒤 방을 닫는다.
/// 60초 유예보다 빠르므로 여기서 끝내지 않고 유예만 시작한다(송출자가 돌아오면 방은 다시 만들어진다).
async fn room_finished(state: &AppState, id: i64) -> Result<(), AppError> {
    if let Some(count) = state.live.update_viewers(id, |s| s.clear()) {
        state
            .realtime
            .publish(id, Event::public(WsEvent::ViewerCount { count }));
    }
    if let Some(since) = db::start_grace(&state.db, id, None).await? {
        tracing::info!(
            broadcast_id = id,
            "livekit room finished while live, grace started"
        );
        state
            .realtime
            .publish(id, Event::public(WsEvent::BroadcastReconnecting { since }));
    }
    Ok(())
}
