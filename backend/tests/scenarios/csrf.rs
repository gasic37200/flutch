//! 변경 요청 Origin 검사 (features/10 보안 공통). 아직 POST 라우트가 없어 /health로 확인한다:
//! 검사를 통과하면 405(메서드 없음), 막히면 403.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use sqlx::PgPool;

use crate::common::{ORIGIN, send, test_app};

fn post(path: &str, origin: Option<&str>) -> Request<Body> {
    let mut req = Request::post(path);
    if let Some(o) = origin {
        req = req.header("origin", o);
    }
    req.body(Body::empty()).unwrap()
}

#[sqlx::test]
async fn mutating_request_without_or_with_foreign_origin_is_forbidden(db: PgPool) {
    let app = test_app(db);

    for origin in [None, Some("https://evil.example")] {
        let res = send(&app, post("/api/v1/health", origin)).await;
        assert_eq!(res.status, StatusCode::FORBIDDEN, "origin {origin:?}");
        assert_eq!(res.json["error"]["code"], "FORBIDDEN");
    }
}

#[sqlx::test]
async fn mutating_request_with_allowed_origin_passes(db: PgPool) {
    let app = test_app(db);

    let res = send(&app, post("/api/v1/health", Some(ORIGIN))).await;

    assert_eq!(res.status, StatusCode::METHOD_NOT_ALLOWED);
}

#[sqlx::test]
async fn webhooks_skip_origin_check(db: PgPool) {
    let app = test_app(db);

    for path in ["/api/v1/livekit/webhook", "/api/v1/payments/webhook"] {
        let res = send(&app, post(path, None)).await;
        assert_ne!(res.status, StatusCode::FORBIDDEN, "{path}");
    }
}

#[sqlx::test]
async fn read_request_needs_no_origin(db: PgPool) {
    let app = test_app(db);

    let res = send(
        &app,
        Request::get("/api/v1/health").body(Body::empty()).unwrap(),
    )
    .await;

    assert_eq!(res.status, StatusCode::OK);
}
