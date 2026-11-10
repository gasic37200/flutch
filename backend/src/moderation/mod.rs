//! 모더레이션(밴) 도메인 — docs/features/05-moderation.md
//! 차단 상태 조회(`chat::ban_state`)와 채팅 쪽 강제는 `chat`에 있다.

use std::sync::Arc;

use axum::Router;
use axum::http::StatusCode;
use axum::routing::{get, put};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::AppState;
use crate::auth::UserSummary;
use crate::error::AppError;

mod db;
mod handlers;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/users/{id}/bans", get(handlers::list))
        .route(
            "/users/{id}/bans/{user_id}",
            put(handlers::put).delete(handlers::delete),
        )
        .route(
            "/users/{id}/viewers/{user_id}/chats",
            get(handlers::viewer_chats),
        )
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Ban {
    pub user: UserSummary,
    pub permanent: bool,
    /// 영구 밴이면 null
    pub until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Deserialize, TS)]
#[ts(export)]
pub enum BanDuration {
    #[serde(rename = "1m")]
    #[ts(rename = "1m")]
    OneMinute,
    #[serde(rename = "10m")]
    #[ts(rename = "10m")]
    TenMinutes,
    #[serde(rename = "1h")]
    #[ts(rename = "1h")]
    OneHour,
    #[serde(rename = "permanent")]
    #[ts(rename = "permanent")]
    Permanent,
}

impl BanDuration {
    fn until(self) -> Option<DateTime<Utc>> {
        let mins = match self {
            Self::OneMinute => 1,
            Self::TenMinutes => 10,
            Self::OneHour => 60,
            Self::Permanent => return None,
        };
        Some(Utc::now() + chrono::Duration::minutes(mins))
    }
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct BanRequest {
    pub duration: BanDuration,
}

/// `chat.system`의 종류
#[derive(Debug, Clone, Copy, Serialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum SystemKind {
    Timeout,
    Ban,
}

#[derive(Debug, thiserror::Error)]
pub enum ModerationError {
    #[error("내 채널에서만 관리할 수 있어요.")]
    NotOwner,
    #[error("자기 자신은 차단할 수 없어요.")]
    SelfBan,
}

impl From<ModerationError> for AppError {
    fn from(e: ModerationError) -> Self {
        let message = e.to_string();
        let (status, code) = match e {
            ModerationError::NotOwner => (StatusCode::FORBIDDEN, "MODERATION_NOT_OWNER"),
            ModerationError::SelfBan => (StatusCode::UNPROCESSABLE_ENTITY, "MODERATION_SELF"),
        };
        AppError::domain(status, code, message)
    }
}
