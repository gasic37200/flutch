//! 로그인 사용자 추출기. 핸들러 인자로 `CurrentUser`(로그인 필수) 또는 `OptionalUser`를 받는다.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::error::AppError;

pub struct CurrentUser {
    pub id: i64,
}

pub struct OptionalUser(pub Option<CurrentUser>);

impl<S: Send + Sync> FromRequestParts<S> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        OptionalUser::from_request_parts(parts, state)
            .await?
            .0
            .ok_or(AppError::Unauthenticated)
    }
}

impl<S: Send + Sync> FromRequestParts<S> for OptionalUser {
    type Rejection = AppError;

    // shortcut: 세션 조회 미구현이라 항상 비로그인, T03에서 `flutch_session` 쿠키 → sessions 조회로 교체
    async fn from_request_parts(_parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(None))
    }
}
