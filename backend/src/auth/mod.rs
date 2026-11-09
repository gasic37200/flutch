//! 인증·계정 도메인 — docs/features/01-accounts.md

use std::sync::Arc;
use std::time::Duration;

use axum::http::StatusCode;
use axum::routing::{get, post, put};
use axum::{Extension, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::AppState;
use crate::error::AppError;
use crate::scheduler::Scheduler;

mod db;
mod handlers;
mod rate_limit;
pub mod rules;
pub(crate) mod session;

pub use rules::{contains_banned_word, contains_profanity};

const DAY: Duration = Duration::from_secs(24 * 60 * 60);

/// 테스트 계정 `(loginId, 닉네임)`. 비밀번호는 `SEED_PASSWORD`, 테스트 로그인은 이 3개만.
pub const SEED_ACCOUNTS: [(&str, &str); 3] = [
    ("streamer1", "스트리머1"),
    ("streamer2", "스트리머2"),
    ("viewer", "시청자"),
];

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth/signup", post(handlers::signup))
        .route("/auth/login", post(handlers::login))
        .route("/auth/test-login", post(handlers::test_login))
        .route("/auth/logout", post(handlers::logout))
        .route("/me", get(handlers::me).patch(handlers::update_me))
        .route("/me/password", put(handlers::change_password))
        .layer(Extension(Arc::new(handlers::Limits::default())))
}

/// 주기 작업: 테스트 계정 생성(시작 직후 1회, 이후 매일 확인), 만료 세션 정리(1일).
pub fn register(s: &mut Scheduler) {
    s.every("auth.seed_accounts", DAY, |state| async move {
        ensure_seed_accounts(&state).await
    });
    s.every("auth.session_cleanup", DAY, |state| async move {
        let deleted = db::delete_expired_sessions(&state.db).await?;
        tracing::info!(deleted, "expired sessions removed");
        Ok(())
    });
}

/// 없는 테스트 계정만 만든다. 서버 시작 시(스케줄러 첫 실행)와 테스트 로그인에서 부른다.
pub async fn ensure_seed_accounts(state: &AppState) -> Result<(), AppError> {
    let ids = SEED_ACCOUNTS.map(|(id, _)| id.to_owned());
    let existing = db::existing_login_ids(&state.db, &ids).await?;
    for (login_id, nickname) in SEED_ACCOUNTS {
        if existing.iter().any(|e| e == login_id) {
            continue;
        }
        // SEED_PASSWORD가 없으면 아무도 모르는 비밀번호 → 테스트 로그인 버튼으로만 들어간다.
        let password = match &state.config.auth.seed_password {
            Some(p) => p.clone(),
            None => session::new_token()?,
        };
        let hash = session::hash_password(password).await?;
        match db::insert_user(&state.db, login_id, &hash, nickname).await {
            Ok(_) => tracing::info!(login_id, "seed account created"),
            Err(e) => tracing::warn!(login_id, error = %e, "seed account not created"),
        }
    }
    Ok(())
}

#[derive(Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UserSummary {
    pub id: i64,
    /// 탈퇴 사용자는 "탈퇴한 사용자"
    pub nickname: String,
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Me {
    pub id: i64,
    /// 카카오 계정은 null
    pub login_id: Option<String>,
    pub nickname: String,
    /// 이 시각 전에는 닉네임 변경 불가(null = 지금 가능)
    pub nickname_changeable_at: Option<DateTime<Utc>>,
    /// false(카카오)면 비밀번호 변경 메뉴 숨김
    pub has_password: bool,
    pub balance: i64,
    /// 내가 지금 LIVE 중인 방송
    pub live_broadcast_id: Option<i64>,
}

#[derive(Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SignupRequest {
    pub login_id: String,
    pub password: String,
    pub nickname: String,
    pub agree_terms: bool,
    pub agree_privacy: bool,
    pub over14: bool,
}

#[derive(Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LoginRequest {
    pub login_id: String,
    pub password: String,
    pub remember: bool,
}

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct TestLoginRequest {
    #[ts(type = "'streamer1' | 'streamer2' | 'viewer'")]
    pub account: String,
}

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct UpdateMeRequest {
    pub nickname: String,
}

#[derive(Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("이미 사용 중인 아이디예요.")]
    LoginIdTaken,
    #[error("이미 사용 중인 닉네임이에요.")]
    NicknameTaken,
    #[error("아이디 또는 비밀번호가 올바르지 않아요.")]
    InvalidCredentials,
    /// 남은 초
    #[error("로그인 시도가 많아 잠겼어요. {}분 후 다시 시도해 주세요.", (.0 + 59) / 60)]
    Locked(i64),
    /// 남은 일수
    #[error("닉네임은 {0}일 후에 바꿀 수 있어요.")]
    NicknameChangeTooSoon(i64),
    #[error("현재 비밀번호가 올바르지 않아요.")]
    WrongPassword,
}

impl From<AuthError> for AppError {
    fn from(e: AuthError) -> Self {
        let message = e.to_string();
        match e {
            AuthError::LoginIdTaken => {
                AppError::domain(StatusCode::CONFLICT, "AUTH_LOGIN_ID_TAKEN", message)
            }
            AuthError::NicknameTaken => {
                AppError::domain(StatusCode::CONFLICT, "AUTH_NICKNAME_TAKEN", message)
            }
            AuthError::InvalidCredentials => AppError::domain(
                StatusCode::UNAUTHORIZED,
                "AUTH_INVALID_CREDENTIALS",
                message,
            ),
            AuthError::Locked(sec) => {
                AppError::domain(StatusCode::TOO_MANY_REQUESTS, "AUTH_LOCKED", message)
                    .with_retry_after(sec)
            }
            AuthError::NicknameChangeTooSoon(_) => AppError::domain(
                StatusCode::CONFLICT,
                "AUTH_NICKNAME_CHANGE_TOO_SOON",
                message,
            ),
            AuthError::WrongPassword => AppError::domain(
                StatusCode::UNPROCESSABLE_ENTITY,
                "AUTH_WRONG_PASSWORD",
                message,
            ),
        }
    }
}
