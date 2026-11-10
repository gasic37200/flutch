//! 방송 도메인 — docs/features/02-broadcasts.md

use std::sync::Arc;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::http::StatusCode;
use axum::routing::{get, put};
use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use ts_rs::TS;

use crate::AppState;
use crate::auth::UserSummary;
use crate::clutch::ClutchGauge;
use crate::error::AppError;
use crate::extract::CurrentUser;
use crate::realtime::{Event, WsEvent};

mod db;
mod handlers;

/// 사용자당 READY 최대 개수.
pub const BROADCAST_READY_LIMIT: i64 = 5;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/broadcasts",
            get(handlers::list_live).post(handlers::create),
        )
        .route(
            "/broadcasts/{id}",
            get(handlers::get_one)
                .patch(handlers::update)
                .delete(handlers::delete),
        )
        .route(
            "/broadcasts/{id}/thumbnail",
            put(handlers::put_thumbnail).layer(DefaultBodyLimit::max(1024 * 1024)),
        )
        .route("/me/broadcasts", get(handlers::list_mine))
        .route("/users/{id}/broadcasts", get(handlers::list_by_user))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Category {
    JustChatting,
    Game,
    Fps,
    Rpg,
    Horror,
    Virtual,
    Sports,
    Music,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum PipCorner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "UPPERCASE")]
#[ts(export)]
pub enum BroadcastStatus {
    Ready,
    Live,
    Ended,
}

/// `broadcast.ended` 이벤트의 reason (api.md §11).
#[derive(Debug, Clone, Copy, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum EndReason {
    Owner,
    Disconnect,
    MaxDuration,
    Deleted,
}

/// `broadcast.ended` 이벤트 data (api.md §11).
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BroadcastEnded {
    pub ended_at: DateTime<Utc>,
    pub peak_viewers: i32,
    pub duration_sec: i64,
    pub reason: EndReason,
    pub recording: Option<RecordingRef>,
}

/// 녹화 상태(api.md §1). `Broadcast.recording`·`recording.updated`가 쓴다.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum RecordingStatus {
    Recording,
    Ready,
    Failed,
    Deleted,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RecordingRef {
    pub id: i64,
    pub status: RecordingStatus,
}

/// 커서 페이지 응답. 다른 도메인의 목록도 이 타입을 쓴다.
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
}

#[derive(Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Broadcast {
    pub id: i64,
    pub owner: UserSummary,
    pub status: BroadcastStatus,
    pub title: String,
    pub category: Category,
    pub tags: Vec<String>,
    pub record: bool,
    pub pip_corner: PipCorner,
    /// `/media/thumbs/{id}.jpg?v=<버전>`, 없으면 null(기본 이미지)
    pub thumbnail_url: Option<String>,
    /// LIVE만 의미(사용자 단위, 소유자 제외), 그 외 0
    pub viewer_count: i64,
    pub peak_viewers: i32,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub reconnecting: bool,
    /// SUPER CLUTCH 진행 중이면 종료 시각
    pub super_clutch_until: Option<DateTime<Utc>>,
    /// record=false거나 아직 없으면 null
    pub recording: Option<RecordingRef>,
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BroadcastDetail {
    #[serde(flatten)]
    #[ts(flatten)]
    pub broadcast: Broadcast,
    /// LIVE가 아니면 percent 0·charging
    pub gauge: ClutchGauge,
    pub burst_price: i64,
    /// 비로그인은 null
    pub viewer: Option<ViewerState>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ViewerState {
    pub is_owner: bool,
    /// 이 스트리머에게 차단된 상태(풀린 임시차단은 null)
    pub ban: Option<BanState>,
    /// 최근 30초 창에서 남은 무료 클러치(0~5)
    pub free_clutches_left: i32,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BanState {
    pub permanent: bool,
    pub until: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CreateBroadcastRequest {
    pub title: String,
    pub category: Category,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_record")]
    pub record: bool,
}

fn default_record() -> bool {
    true
}

#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UpdateBroadcastRequest {
    pub title: Option<String>,
    pub category: Option<Category>,
    pub tags: Option<Vec<String>>,
    pub record: Option<bool>,
    pub pip_corner: Option<PipCorner>,
}

#[derive(Debug, thiserror::Error)]
pub enum BroadcastError {
    #[error("방송 소유자만 할 수 있어요.")]
    NotOwner,
    #[error("지금 방송 상태에서는 할 수 없어요.")]
    InvalidTransition,
    #[error("이미 진행 중인 방송이 있어요.")]
    LiveLimit,
    #[error("준비 중인 방송은 최대 5개까지 만들 수 있어요.")]
    ReadyLimit,
    #[error("지금 방송 상태에서는 이 항목을 바꿀 수 없어요.")]
    FieldLocked,
    #[error("썸네일은 512KB 이하 JPEG만 올릴 수 있어요.")]
    ThumbnailInvalid,
}

impl From<BroadcastError> for AppError {
    fn from(e: BroadcastError) -> Self {
        let (status, code) = match e {
            BroadcastError::NotOwner => (StatusCode::FORBIDDEN, "BROADCAST_NOT_OWNER"),
            BroadcastError::InvalidTransition => {
                (StatusCode::CONFLICT, "BROADCAST_INVALID_TRANSITION")
            }
            BroadcastError::LiveLimit => (StatusCode::CONFLICT, "BROADCAST_LIVE_LIMIT"),
            BroadcastError::ReadyLimit => (StatusCode::CONFLICT, "BROADCAST_READY_LIMIT"),
            BroadcastError::FieldLocked => (StatusCode::CONFLICT, "BROADCAST_FIELD_LOCKED"),
            BroadcastError::ThumbnailInvalid => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "BROADCAST_THUMBNAIL_INVALID",
            ),
        };
        AppError::domain(status, code, e.to_string())
    }
}

/// LiveKit 방 이름.
pub fn room_name(id: i64) -> String {
    format!("broadcast-{id}")
}

/// DB 문자열 → enum. 스키마 CHECK가 보장하므로 실패하면 서버 오류.
fn parse_enum<T: DeserializeOwned>(s: &str) -> Result<T, AppError> {
    serde_json::from_value(serde_json::Value::String(s.to_owned()))
        .map_err(|_| AppError::Internal(format!("unexpected enum value {s:?}")))
}

/// enum → DB 문자열.
fn enum_str<T: Serialize>(v: &T) -> Result<String, AppError> {
    match serde_json::to_value(v) {
        Ok(serde_json::Value::String(s)) => Ok(s),
        _ => Err(AppError::Internal("enum is not a string".into())),
    }
}

/// 상태 전이 `READY → LIVE`. 소유자 확인은 호출자 몫. 소유자의 다른 LIVE가 있으면 `BROADCAST_LIVE_LIMIT`.
pub async fn start_live(db: &PgPool, id: i64) -> Result<(), AppError> {
    match db::mark_live(db, id).await {
        Ok(true) => {
            tracing::info!(broadcast_id = id, "broadcast started");
            Ok(())
        }
        Ok(false) => Err(missing_or_invalid(db, id).await),
        Err(e) if db::is_live_limit(&e) => Err(BroadcastError::LiveLimit.into()),
        Err(e) => Err(e.into()),
    }
}

/// 종료 공통 절차: `LIVE → ENDED` → LiveKit 방 삭제 → `broadcast.ended`.
/// LiveKit 실패는 종료를 되돌리지 않는다(종료가 우선, 로그만).
/// `Disconnect`는 유예 중일 때만 끝낸다(만료 판정과 종료 사이에 송출자가 돌아왔으면 그대로 두고 `Ok`).
pub async fn end_live(state: &AppState, id: i64, reason: EndReason) -> Result<(), AppError> {
    let disconnect = matches!(reason, EndReason::Disconnect);
    if !db::mark_ended(&state.db, id, disconnect).await? {
        if disconnect {
            tracing::info!(
                broadcast_id = id,
                "publisher back before grace end, not ending"
            );
            return Ok(());
        }
        return Err(missing_or_invalid(&state.db, id).await);
    }
    // Egress를 먼저 멈춰 파일을 마무리시킨 뒤 방을 닫는다. 실패해도 종료는 유지.
    crate::recording::stop(state, id).await;
    if let Err(e) = state.livekit.delete_room(&room_name(id)).await {
        tracing::error!(broadcast_id = id, error = %e, "livekit room delete failed");
    }
    tracing::info!(broadcast_id = id, ?reason, "broadcast ended");
    // 종료는 이미 커밋됐다. 이벤트를 못 만들어도 종료를 실패로 돌리지 않는다(시청자는 재조회로 따라온다).
    match ended_event(state, id, reason).await {
        Ok(data) => state
            .realtime
            .publish(id, Event::public(WsEvent::BroadcastEnded(data))),
        Err(e) => tracing::error!(broadcast_id = id, error = %e, "broadcast.ended event failed"),
    }
    Ok(())
}

async fn ended_event(
    state: &AppState,
    id: i64,
    reason: EndReason,
) -> Result<BroadcastEnded, AppError> {
    let row = handlers::load(state, id).await?;
    let ended_at = row
        .ended_at
        .ok_or_else(|| AppError::Internal("ended broadcast without ended_at".into()))?;
    Ok(BroadcastEnded {
        ended_at,
        peak_viewers: row.peak_viewers,
        duration_sec: row.started_at.map_or(0, |s| (ended_at - s).num_seconds()),
        reason,
        recording: crate::recording::ref_of(state, id).await?,
    })
}

/// `GET /broadcasts/{id}`와 같은 상세 응답. 다른 도메인(live) 응답에 싣는다. 숨김·없음은 404.
pub async fn detail_of(
    state: &AppState,
    id: i64,
    viewer: Option<&CurrentUser>,
) -> Result<BroadcastDetail, AppError> {
    handlers::detail(state, handlers::load(state, id).await?, viewer).await
}

async fn missing_or_invalid(db: &PgPool, id: i64) -> AppError {
    match db::status_of(db, id).await {
        Ok(Some(_)) => BroadcastError::InvalidTransition.into(),
        Ok(None) => AppError::NotFound,
        Err(e) => e.into(),
    }
}

/// 이 사용자들의 LIVE·미숨김 방송. PULSE용(LIVE 방송은 전체가 적어 전부 읽어 거른다).
pub async fn live_of_owners(state: &AppState, owners: &[i64]) -> Result<Vec<Broadcast>, AppError> {
    let f = db::Filter {
        status: Some("LIVE"),
        limit: i64::MAX,
        ..db::Filter::default()
    };
    db::find(&state.db, &f)
        .await?
        .into_iter()
        .filter(|r| owners.contains(&r.owner_id))
        .map(|r| handlers::into_broadcast(state, r))
        .collect()
}

/// 모든 LIVE·미숨김 방송(최신 id 순). 홈 히어로용.
pub async fn live_all(state: &AppState) -> Result<Vec<Broadcast>, AppError> {
    let f = db::Filter {
        status: Some("LIVE"),
        limit: i64::MAX,
        ..db::Filter::default()
    };
    db::find(&state.db, &f)
        .await?
        .into_iter()
        .map(|r| handlers::into_broadcast(state, r))
        .collect()
}

/// 그 사용자의 LIVE·미숨김 방송(없으면 `None`). 스트리머 페이지용.
pub async fn live_of_owner(state: &AppState, owner_id: i64) -> Result<Option<Broadcast>, AppError> {
    let f = db::Filter {
        owner_id: Some(owner_id),
        status: Some("LIVE"),
        limit: 1,
        ..db::Filter::default()
    };
    db::find(&state.db, &f)
        .await?
        .into_iter()
        .next()
        .map(|r| handlers::into_broadcast(state, r))
        .transpose()
}
