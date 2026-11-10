use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;

use crate::common::{send, test_app};

#[sqlx::test]
async fn health_check_reports_ok_with_request_id(db: PgPool) {
    let app = test_app(db);

    let res = send(
        &app,
        Request::get("/api/v1/health").body(Body::empty()).unwrap(),
    )
    .await;

    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.json, serde_json::json!({ "status": "ok" }));
    assert!(res.headers.contains_key("x-request-id"));
}

#[tokio::test]
async fn health_check_reports_503_when_db_is_down() {
    let db = PgPoolOptions::new()
        .acquire_timeout(Duration::from_millis(300))
        .connect_lazy("postgres://flutch:flutch@127.0.0.1:1/none")
        .unwrap();
    let app = test_app(db);

    let res = send(
        &app,
        Request::get("/api/v1/health").body(Body::empty()).unwrap(),
    )
    .await;

    assert_eq!(res.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(res.json["status"], "unavailable");
}

#[sqlx::test]
async fn unknown_api_path_returns_not_found_error_format(db: PgPool) {
    let app = test_app(db);

    let res = send(
        &app,
        Request::get("/api/v1/nope").body(Body::empty()).unwrap(),
    )
    .await;

    assert_eq!(res.status, StatusCode::NOT_FOUND);
    assert_eq!(res.json["error"]["code"], "NOT_FOUND");
    assert_eq!(
        res.json["error"]["requestId"],
        res.headers["x-request-id"].to_str().unwrap()
    );
}

#[sqlx::test]
async fn metrics_endpoint_exposes_request_counter(db: PgPool) {
    let app = test_app(db);
    send(
        &app,
        Request::get("/api/v1/health").body(Body::empty()).unwrap(),
    )
    .await;

    let res = app
        .clone()
        .oneshot(Request::get("/metrics").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let text = String::from_utf8(body.to_vec()).unwrap();

    // 경로 id가 아닌 route 템플릿으로 센다
    assert!(
        text.contains("http_requests_total{") && text.contains("route=\"/api/v1/health\""),
        "{text}"
    );
    assert!(
        text.contains("http_request_duration_seconds_bucket"),
        "{text}"
    );
    assert!(text.contains("live_broadcasts 0"), "{text}");
    assert!(text.contains("ws_connections 0"), "{text}");
}

#[sqlx::test]
async fn metrics_method_label_is_bounded(db: PgPool) {
    let app = test_app(db);
    send(
        &app,
        Request::builder()
            .method("BREW")
            .uri("/api/v1/health")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    let res = app
        .clone()
        .oneshot(Request::get("/metrics").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let text = String::from_utf8(body.to_vec()).unwrap();
    assert!(text.contains("method=\"OTHER\""), "{text}");
    assert!(!text.contains("BREW"), "{text}");
}
