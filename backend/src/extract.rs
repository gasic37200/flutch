//! 요청 추출기.
//! - `CurrentUser`: 로그인 필수(없으면 401). `OptionalUser`: 비로그인이면 `None`.
//! - `JsonBody<T>`: `axum::Json` 대신 쓴다. 본문이 형식에 안 맞으면 api.md 형식의 422로 답한다.

use std::collections::BTreeMap;
use std::sync::Arc;

use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRequest, FromRequestParts};
use axum::http::request::Parts;

use crate::AppState;
use crate::auth::session;
use crate::error::AppError;

/// `flutch_session` 쿠키로 찾은 로그인 사용자. 요청 로그 span의 `user_id`도 채운다.
pub struct CurrentUser {
    pub id: i64,
    pub remember: bool,
    /// 지금 요청의 세션(비밀번호 변경 때 이 세션만 남긴다).
    pub(crate) session_hash: [u8; 32],
}

pub struct OptionalUser(pub Option<CurrentUser>);

impl FromRequestParts<Arc<AppState>> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        OptionalUser::from_request_parts(parts, state)
            .await?
            .0
            .ok_or(AppError::Unauthenticated)
    }
}

impl FromRequestParts<Arc<AppState>> for OptionalUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let Some(token) = session::token(&parts.headers) else {
            return Ok(Self(None));
        };
        let user = session::authenticate(&state.db, token).await?;
        if let Some(u) = &user {
            tracing::Span::current().record("user_id", u.id);
        }
        Ok(Self(user))
    }
}

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(AppError))]
pub struct JsonBody<T>(pub T);

impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> Self {
        tracing::debug!(error = %rejection, "invalid json body");
        AppError::Validation(BTreeMap::from([(
            "body".to_owned(),
            "요청 형식이 올바르지 않습니다.".to_owned(),
        )]))
    }
}
