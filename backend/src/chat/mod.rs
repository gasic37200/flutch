//! 채팅 도메인 — docs/features/04-chat.md

use std::sync::Arc;
use std::time::Duration;

use axum::http::StatusCode;
use axum::routing::{get, patch};
use axum::{Extension, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use ts_rs::TS;

use crate::AppState;
use crate::auth::UserSummary;
use crate::auth::rate_limit::RateLimiter;
use crate::broadcast::BanState;
use crate::error::AppError;

pub(crate) mod db;
mod handlers;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/broadcasts/{id}/chats",
            get(handlers::list).post(handlers::create),
        )
        .route(
            "/chats/{id}",
            patch(handlers::update).delete(handlers::delete),
        )
        .layer(Extension(Arc::new(Limits::default())))
}

/// 도배 제한(사용자·방송 단위 1초 1개). 메모리라 서버를 다시 띄우면 비워진다.
pub(crate) struct Limits {
    send: RateLimiter,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            send: RateLimiter::new(1, Duration::from_secs(1)),
        }
    }
}

impl Limits {
    fn hit(&self, user_id: i64, broadcast_id: i64) -> Result<(), AppError> {
        self.send.hit(&format!("{user_id}:{broadcast_id}"))
    }
}

#[derive(Debug, Clone, Copy, Serialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum Badge {
    Streamer,
    Donor,
}

#[derive(Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Chat {
    pub id: i64,
    pub broadcast_id: i64,
    pub author: UserSummary,
    /// 욕설은 `***`로 가린 결과. 삭제됐으면 null
    pub body: Option<String>,
    /// "(수정됨)"
    pub edited: bool,
    /// "삭제된 메시지입니다"
    pub deleted: bool,
    /// donor = 이 방송에 donation·burst 후원 이력(익명 후원 제외 — 신원 노출 방지, 사용자 승인 11/18)
    pub badges: Vec<Badge>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ChatPage {
    /// 오래된 것 → 최신
    pub items: Vec<Chat>,
    pub has_more: bool,
}

/// 작성(POST)·수정(PATCH) 요청 본문.
#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct ChatBodyRequest {
    pub body: String,
}

/// 그 스트리머가 이 사용자에게 건 지금 유효한 차단. 없으면 `None`(풀린 임시차단 포함).
/// 채팅이 쓰고, 후원·클러치(영구 밴만 막는다)도 이 함수로 확인한다.
/// 트랜잭션 안에서는 그 연결(`&mut *tx`)을 넘긴다 — 풀에서 두 번째 연결을 잡으면 동시 요청이 풀을 다 써 교착된다.
pub async fn ban_state(
    db: impl sqlx::PgExecutor<'_>,
    streamer_id: i64,
    user_id: i64,
) -> Result<Option<BanState>, sqlx::Error> {
    Ok(db::active_ban(db, streamer_id, user_id)
        .await?
        .map(|until| BanState {
            permanent: until.is_none(),
            until,
        }))
}

/// 채팅 작성·수정 전 차단 확인: 임시차단 `MODERATION_TIMED_OUT`(+`retryAfterSec`), 영구 `MODERATION_BANNED`.
pub async fn ensure_can_chat(db: &PgPool, streamer_id: i64, user_id: i64) -> Result<(), AppError> {
    match ban_state(db, streamer_id, user_id).await? {
        None => Ok(()),
        Some(BanState { until: None, .. }) => Err(ChatError::Banned.into()),
        Some(BanState { until: Some(t), .. }) => {
            let ms = (t - Utc::now()).num_milliseconds();
            Err(ChatError::TimedOut((ms + 999) / 1000).into())
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ChatError {
    #[error("방송 중에만 채팅할 수 있어요.")]
    NotLive,
    #[error("FLUTCH 정책상 사용할 수 없는 표현이 포함되어 있어요.")]
    BannedWord,
    #[error("같은 메시지를 반복할 수 없어요.")]
    Repeated,
    #[error("작성자만 수정할 수 있어요.")]
    NotAuthor,
    #[error("작성 후 5분이 지나 수정할 수 없어요.")]
    EditExpired,
    #[error("삭제된 메시지예요.")]
    Deleted,
    #[error("작성자 또는 방송 소유자만 삭제할 수 있어요.")]
    DeleteForbidden,
    #[error("이 채널에서 채팅이 제한되었어요.")]
    Banned,
    /// 남은 초
    #[error("{}분 후 채팅할 수 있어요.", ((.0 + 59) / 60).max(1))]
    TimedOut(i64),
}

impl From<ChatError> for AppError {
    fn from(e: ChatError) -> Self {
        let message = e.to_string();
        let (status, code) = match e {
            ChatError::NotLive => (StatusCode::CONFLICT, "BROADCAST_NOT_LIVE"),
            ChatError::BannedWord => (StatusCode::UNPROCESSABLE_ENTITY, "CHAT_BANNED_WORD"),
            ChatError::Repeated => (StatusCode::TOO_MANY_REQUESTS, "CHAT_REPEATED"),
            ChatError::NotAuthor => (StatusCode::FORBIDDEN, "CHAT_NOT_AUTHOR"),
            ChatError::EditExpired => (StatusCode::CONFLICT, "CHAT_EDIT_EXPIRED"),
            ChatError::Deleted => (StatusCode::CONFLICT, "CHAT_DELETED"),
            ChatError::DeleteForbidden => (StatusCode::FORBIDDEN, "CHAT_DELETE_FORBIDDEN"),
            ChatError::Banned => (StatusCode::FORBIDDEN, "MODERATION_BANNED"),
            ChatError::TimedOut(sec) => {
                return AppError::domain(StatusCode::FORBIDDEN, "MODERATION_TIMED_OUT", message)
                    .with_retry_after(sec);
            }
        };
        AppError::domain(status, code, message)
    }
}
