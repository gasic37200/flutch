//! 공통 미들웨어: 요청 ID·접근 로그, 변경 요청 `Origin` 검사(CSRF).

use std::sync::Arc;
use std::time::Instant;

use axum::extract::{MatchedPath, OriginalUri, Request, State};
use axum::http::{HeaderMap, HeaderValue, Method, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use tracing::Instrument;

use crate::AppState;
use crate::error::AppError;

tokio::task_local! {
    static REQUEST_ID: String;
}

/// 지금 처리 중인 요청의 ID. 요청 밖(주기 작업 등)에서는 빈 문자열.
pub fn current_request_id() -> String {
    REQUEST_ID.try_with(Clone::clone).unwrap_or_default()
}

/// 요청마다 ID를 만들어 에러 본문·`x-request-id` 헤더·로그 span에 같이 싣는다.
/// `user_id`는 비워 두고 인증 추출기가 `Span::current().record("user_id", id)`로 채운다.
pub async fn request_context(req: Request, next: Next) -> Response {
    let id = uuid::Uuid::new_v4().to_string();
    let route = req
        .extensions()
        .get::<MatchedPath>()
        .map_or("unmatched", MatchedPath::as_str)
        .to_owned();
    // 임의 메서드 문자열이 메트릭 라벨(시리즈)이 되지 않게 표준 메서드만 쓴다.
    let req_method = match req.method().as_str() {
        m @ ("GET" | "HEAD" | "POST" | "PUT" | "PATCH" | "DELETE" | "OPTIONS") => m,
        _ => "OTHER",
    }
    .to_owned();
    let span = tracing::info_span!(
        "request",
        request_id = %id,
        method = %req.method(),
        route = %route,
        user_id = tracing::field::Empty,
    );
    let started = Instant::now();
    let mut res = REQUEST_ID
        .scope(id.clone(), next.run(req))
        .instrument(span.clone())
        .await;
    crate::metrics::record_request(
        &req_method,
        &route,
        res.status().as_u16(),
        started.elapsed().as_secs_f64(),
    );
    span.in_scope(|| {
        tracing::info!(
            status = res.status().as_u16(),
            latency_ms = started.elapsed().as_millis() as u64,
            "request"
        )
    });
    if let Ok(v) = HeaderValue::from_str(&id) {
        res.headers_mut().insert("x-request-id", v);
    }
    res
}

/// 웹훅은 서버 간 호출이라 Origin이 없다. 대신 서명(LiveKit)·결제 재조회(토스)로 검증한다.
const ORIGIN_EXEMPT: [&str; 2] = ["/api/v1/livekit/webhook", "/api/v1/payments/webhook"];

/// 변경 요청(POST/PUT/PATCH/DELETE)은 `Origin`이 `ALLOWED_ORIGINS`에 있어야 한다. 없거나 다르면 403.
pub async fn check_origin(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Response {
    let mutating = matches!(
        *req.method(),
        Method::POST | Method::PUT | Method::PATCH | Method::DELETE
    );
    // nest 안에서도 전체 경로로 비교한다.
    let path = req
        .extensions()
        .get::<OriginalUri>()
        .map_or(req.uri().path(), |u| u.path());
    if mutating
        && !ORIGIN_EXEMPT.contains(&path)
        && !origin_allowed(req.headers(), &state.config.allowed_origins)
    {
        tracing::warn!(path, "origin rejected");
        return AppError::Forbidden.into_response();
    }
    next.run(req).await
}

/// `Origin` 헤더가 허용 목록에 있는지. 없으면 false. WebSocket 업그레이드(GET)도 이걸로 검사한다.
pub fn origin_allowed(headers: &HeaderMap, allowed: &[String]) -> bool {
    headers
        .get(header::ORIGIN)
        .and_then(|o| o.to_str().ok())
        .is_some_and(|o| allowed.iter().any(|a| a == o))
}
