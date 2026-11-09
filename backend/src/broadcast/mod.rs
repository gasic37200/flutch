//! 방송 도메인 — docs/features/02-broadcasts.md

use std::sync::Arc;

use axum::Router;
use axum::http::StatusCode;
use axum::routing::get;
use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use ts_rs::TS;

use crate::AppState;
use crate::auth::UserSummary;
use crate::error::AppError;
use crate::extract::CurrentUser;

mod db;
mod handlers;

/// 사용자당 READY 최대 개수.
pub const BROADCAST_READY_LIMIT: i64 = 5;
/// SUPER BURST 가격. shortcut: 상수, T24가 설정값으로 바꾼다.
const BURST_PRICE: i64 = 1000;

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
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EndReason {
    Owner,
    Disconnect,
    MaxDuration,
    Deleted,
}

/// shortcut: 녹화 task(T25)가 recording 도메인으로 옮긴다. 지금은 `Broadcast.recording` 타입용.
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

/// shortcut: 클러치 task(T19)가 clutch 도메인으로 옮긴다. 지금은 `BroadcastDetail.gauge` 타입용.
#[derive(Debug, Clone, Copy, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ClutchPhase {
    Charging,
    Imminent,
    Cooldown,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ClutchGauge {
    /// 0~100 정수
    pub percent: i32,
    pub phase: ClutchPhase,
    pub cooldown_until: Option<DateTime<Utc>>,
}

/// 커서 페이지 응답. 다른 도메인의 목록도 이 타입을 쓴다.
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
}

#[derive(Serialize, TS)]
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
    /// LIVE만 의미. shortcut: 항상 0, T08b가 시청자 수를 채운다(`db::Row::into_broadcast`).
    pub viewer_count: i64,
    pub peak_viewers: i32,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub reconnecting: bool,
    /// shortcut: 항상 null, T19/T29가 채운다.
    pub super_clutch_until: Option<DateTime<Utc>>,
    /// shortcut: 항상 null, T25가 채운다.
    pub recording: Option<RecordingRef>,
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BroadcastDetail {
    #[serde(flatten)]
    #[ts(flatten)]
    pub broadcast: Broadcast,
    /// shortcut: 항상 percent 0·charging, T19가 채운다(`handlers::detail`).
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
    /// shortcut: 항상 null, T18이 채운다.
    pub ban: Option<BanState>,
    /// shortcut: 항상 5, T19가 채운다.
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
pub async fn end_live(state: &AppState, id: i64, reason: EndReason) -> Result<(), AppError> {
    if !db::mark_ended(&state.db, id).await? {
        return Err(missing_or_invalid(&state.db, id).await);
    }
    if let Err(e) = state.livekit.delete_room(&room_name(id)).await {
        tracing::error!(broadcast_id = id, error = %e, "livekit room delete failed");
    }
    // shortcut: Egress 종료(T25)와 `broadcast.ended` 발행(T08c의 Hub API)은 아직 없다.
    tracing::info!(broadcast_id = id, ?reason, "broadcast ended");
    Ok(())
}

/// `GET /broadcasts/{id}`와 같은 상세 응답. 다른 도메인(live) 응답에 싣는다. 숨김·없음은 404.
pub async fn detail_of(
    state: &AppState,
    id: i64,
    viewer: Option<&CurrentUser>,
) -> Result<BroadcastDetail, AppError> {
    handlers::detail(handlers::load(state, id).await?, viewer)
}

async fn missing_or_invalid(db: &PgPool, id: i64) -> AppError {
    match db::status_of(db, id).await {
        Ok(Some(_)) => BroadcastError::InvalidTransition.into(),
        Ok(None) => AppError::NotFound,
        Err(e) => e.into(),
    }
}
