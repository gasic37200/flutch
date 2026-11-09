use std::collections::BTreeMap;
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};

use super::db::{self, Claim};
use super::{LiveError, LiveKitToken, StartLiveRequest, StartLiveResponse, ViewerTokenRequest};
use crate::AppState;
use crate::broadcast::{self, BroadcastDetail, BroadcastError, EndReason, room_name};
use crate::error::AppError;
use crate::extract::{CurrentUser, JsonBody, OptionalUser};
use crate::livekit;

const ID_FORMAT: &str = "영문·숫자·_·- 8~64자여야 해요.";

/// `tabId`·`guestId` 형식 `^[A-Za-z0-9_-]{8,64}$`.
fn valid_id(s: &str) -> bool {
    (8..=64).contains(&s.len())
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

fn invalid(field: &str) -> AppError {
    AppError::Validation(BTreeMap::from([(field.to_owned(), ID_FORMAT.to_owned())]))
}

/// 소유자 확인 후 현재 상태. 없음·숨김 404, 다른 사용자 403.
async fn owned_status(state: &AppState, id: i64, user: &CurrentUser) -> Result<String, AppError> {
    let t = db::target(&state.db, id).await?.ok_or(AppError::NotFound)?;
    if t.owner_id != user.id {
        return Err(BroadcastError::NotOwner.into());
    }
    Ok(t.status)
}

fn token(
    state: &AppState,
    id: i64,
    identity: String,
    publish: bool,
) -> Result<LiveKitToken, AppError> {
    let room = room_name(id);
    let token = livekit::access_token(
        &state.config.livekit,
        &room,
        &identity,
        publish,
        chrono::Utc::now().timestamp(),
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(LiveKitToken {
        token,
        url: state.config.livekit.url.clone(),
        identity,
        room,
    })
}

/// 송출 시작 또는 이어서 송출(api.md §3-1 상태표).
pub async fn start(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path(id): Path<i64>,
    JsonBody(req): JsonBody<StartLiveRequest>,
) -> Result<Json<StartLiveResponse>, AppError> {
    let status = owned_status(&state, id, &user).await?;
    if !valid_id(&req.tab_id) {
        return Err(invalid("tabId"));
    }
    let identity = format!("u:{}:{}", user.id, req.tab_id);
    if status == "READY" {
        broadcast::start_live(&state.db, id).await?;
        // shortcut: 녹화 켜짐이면 Egress 시작(T25)과 `broadcast.updated`(LIVE) 발행(T08c)이 여기 붙는다.
    }
    let takeover = req.takeover.unwrap_or(false);
    let previous = match db::claim_publisher(&state.db, id, &identity, takeover).await? {
        Claim::NotLive => return Err(BroadcastError::InvalidTransition.into()),
        Claim::Active => return Err(LiveError::PublisherActive.into()),
        Claim::Claimed(previous) => previous,
    };
    if let Some(prev) = previous {
        // 실패해도 새 기기 송출은 진행(로그만): 이전 기기가 잠시 같이 송출될 수 있다.
        match state
            .livekit
            .remove_participant(&room_name(id), &prev)
            .await
        {
            Ok(()) => {
                tracing::info!(broadcast_id = id, from = %prev, to = %identity, "publisher switched")
            }
            Err(e) => {
                tracing::error!(broadcast_id = id, from = %prev, error = %e, "livekit remove publisher failed")
            }
        }
    }
    tracing::info!(broadcast_id = id, identity = %identity, "publish token issued");
    let livekit = token(&state, id, identity, true)?;
    Ok(Json(StartLiveResponse {
        broadcast: broadcast::detail_of(&state, id, Some(&user)).await?,
        livekit,
    }))
}

/// 송출 종료. 상태 전이·방 삭제는 `broadcast::end_live` 공통 절차.
pub async fn end(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<Json<BroadcastDetail>, AppError> {
    owned_status(&state, id, &user).await?;
    broadcast::end_live(&state, id, EndReason::Owner).await?;
    Ok(Json(broadcast::detail_of(&state, id, Some(&user)).await?))
}

/// 시청(구독 전용) 토큰. 신원 `u:<userId>:<tabId>` / `g:<guestId>:<tabId>`.
pub async fn viewer_token(
    State(state): State<Arc<AppState>>,
    OptionalUser(user): OptionalUser,
    Path(id): Path<i64>,
    JsonBody(req): JsonBody<ViewerTokenRequest>,
) -> Result<Json<LiveKitToken>, AppError> {
    let t = db::target(&state.db, id).await?.ok_or(AppError::NotFound)?;
    if !valid_id(&req.tab_id) {
        return Err(invalid("tabId"));
    }
    let who = match (&user, req.guest_id.as_deref()) {
        (Some(u), _) => format!("u:{}", u.id),
        (None, Some(g)) if valid_id(g) => format!("g:{g}"),
        (None, _) => return Err(invalid("guestId")),
    };
    if t.status != "LIVE" {
        return Err(LiveError::NotLive.into());
    }
    Ok(Json(token(
        &state,
        id,
        format!("{who}:{}", req.tab_id),
        false,
    )?))
}
