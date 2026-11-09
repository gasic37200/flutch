//! 공통 에러 → `{error:{code,message,requestId}}` (docs/api.md 규칙).
//!
//! 도메인 에러는 도메인 폴더에 `thiserror` enum으로 만들고 `From<도메인Error> for AppError`에서
//! `AppError::Domain`으로 변환한다. 핸들러는 `?` 한 번으로 올린다. 예시는 아래 `tests` 모듈.

use std::collections::BTreeMap;

use axum::Json;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use ts_rs::TS;

use crate::middleware::current_request_id;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("unauthenticated")]
    Unauthenticated,
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    /// 필드 이름 → 사용자에게 보여줄 이유.
    #[error("validation failed: {0:?}")]
    Validation(BTreeMap<String, String>),
    #[error("rate limited")]
    RateLimited { retry_after_sec: i64 },
    #[error("conflict")]
    Conflict,
    /// 도메인 코드(`<도메인>_<이유>`)와 한국어 문구. `retry_after_sec`는 `with_retry_after`로 싣는다.
    #[error("{code}")]
    Domain {
        status: StatusCode,
        code: &'static str,
        message: String,
        retry_after_sec: Option<i64>,
    },
    #[error("database: {0}")]
    Database(#[from] sqlx::Error),
    #[error("internal: {0}")]
    Internal(String),
}

impl AppError {
    pub fn domain(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self::Domain {
            status,
            code,
            message: message.into(),
            retry_after_sec: None,
        }
    }

    /// 도메인 에러에 `retryAfterSec`·`Retry-After`를 싣는다(예: `AUTH_LOCKED`, `MODERATION_TIMED_OUT`).
    pub fn with_retry_after(mut self, sec: i64) -> Self {
        if let Self::Domain {
            retry_after_sec, ..
        } = &mut self
        {
            *retry_after_sec = Some(sec);
        }
        self
    }
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct ApiError {
    pub error: ErrorBody,
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    pub request_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub fields: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub retry_after_sec: Option<i64>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            Self::Unauthenticated => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHENTICATED",
                "로그인이 필요합니다.".to_string(),
            ),
            Self::Forbidden => (
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                "권한이 없습니다.".to_string(),
            ),
            Self::NotFound | Self::Database(sqlx::Error::RowNotFound) => (
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                "요청한 항목을 찾을 수 없습니다.".to_string(),
            ),
            Self::Validation(_) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "VALIDATION_FAILED",
                "입력값을 확인해 주세요.".to_string(),
            ),
            Self::RateLimited { .. } => (
                StatusCode::TOO_MANY_REQUESTS,
                "RATE_LIMITED",
                "요청이 너무 많습니다. 잠시 후 다시 시도해 주세요.".to_string(),
            ),
            Self::Conflict => (
                StatusCode::CONFLICT,
                "CONFLICT",
                "지금은 처리할 수 없는 상태입니다.".to_string(),
            ),
            Self::Domain {
                status,
                code,
                message,
                ..
            } => (*status, *code, message.clone()),
            Self::Database(_) | Self::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL",
                "일시적인 오류가 발생했습니다. 잠시 후 다시 시도해 주세요.".to_string(),
            ),
        };
        if status.is_server_error() {
            tracing::error!(error = %self, code, "request failed");
        }

        let (fields, retry_after_sec) = match self {
            Self::Validation(fields) => (Some(fields), None),
            Self::RateLimited { retry_after_sec }
            | Self::Domain {
                retry_after_sec: Some(retry_after_sec),
                ..
            } => (None, Some(retry_after_sec)),
            _ => (None, None),
        };
        let body = ApiError {
            error: ErrorBody {
                code: code.to_string(),
                message,
                request_id: current_request_id(),
                fields,
                retry_after_sec,
            },
        };
        let mut res = (status, Json(body)).into_response();
        if let Some(sec) = retry_after_sec {
            res.headers_mut()
                .insert(header::RETRY_AFTER, HeaderValue::from(sec.max(0)));
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 도메인 에러 패턴 예시 — 실제 도메인은 `src/<도메인>/` 안에 같은 모양으로 둔다.
    #[derive(Debug, thiserror::Error)]
    enum ExampleError {
        #[error("이미 종료된 방송입니다.")]
        AlreadyEnded,
    }

    impl From<ExampleError> for AppError {
        fn from(e: ExampleError) -> Self {
            match e {
                ExampleError::AlreadyEnded => {
                    AppError::domain(StatusCode::CONFLICT, "EXAMPLE_ALREADY_ENDED", e.to_string())
                }
            }
        }
    }

    fn handler_like() -> Result<(), AppError> {
        Err(ExampleError::AlreadyEnded)?
    }

    async fn body_json(err: AppError) -> (StatusCode, serde_json::Value) {
        let res = err.into_response();
        let status = res.status();
        let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    #[tokio::test]
    async fn domain_error_maps_to_status_and_code() {
        let (status, json) = body_json(handler_like().unwrap_err()).await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(json["error"]["code"], "EXAMPLE_ALREADY_ENDED");
        assert_eq!(json["error"]["message"], "이미 종료된 방송입니다.");
        assert!(json["error"].get("fields").is_none());
    }

    #[tokio::test]
    async fn validation_and_rate_limit_carry_extra_fields() {
        let fields = BTreeMap::from([("title".to_string(), "1~100자".to_string())]);
        let (status, json) = body_json(AppError::Validation(fields)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(json["error"]["fields"]["title"], "1~100자");

        let (status, json) = body_json(AppError::RateLimited { retry_after_sec: 3 }).await;
        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(json["error"]["retryAfterSec"], 3);

        let err = AppError::domain(StatusCode::FORBIDDEN, "EXAMPLE_WAIT", "기다려 주세요.")
            .with_retry_after(5);
        let (status, json) = body_json(err).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(json["error"]["retryAfterSec"], 5);
        assert!(json["error"]["requestId"].is_string());
    }
}
