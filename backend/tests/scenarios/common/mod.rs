//! 테스트용 앱 생성. DB는 `#[sqlx::test]`가 준 진짜 Postgres, 외부 연동(LiveKit·토스)만 fake.

use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::{HeaderMap, Request, StatusCode};
use flutch::config::{Config, LiveKitConfig};
use flutch::livekit::LiveKitApi;
use flutch::toss::TossApi;
use flutch::{AppState, realtime};
use sqlx::PgPool;
use tower::ServiceExt;

pub const ORIGIN: &str = "http://localhost:5173";

/// trait에 메서드가 생기면 여기서 호출 기록·응답을 조작해 웹훅 반복·실패를 재현한다.
pub struct FakeLiveKit;
impl LiveKitApi for FakeLiveKit {}

pub struct FakeToss;
impl TossApi for FakeToss {}

pub fn test_app(db: PgPool) -> Router {
    let config = Config {
        backend_port: 0,
        database_url: String::new(),
        allowed_origins: vec![ORIGIN.to_string()],
        livekit: LiveKitConfig {
            url: "ws://livekit.test".into(),
            api_key: "devkey".into(),
            api_secret: "secret".into(),
        },
    };
    flutch::app(Arc::new(AppState {
        config,
        db,
        livekit: Arc::new(FakeLiveKit),
        toss: Arc::new(FakeToss),
        realtime: realtime::Hub::default(),
    }))
}

pub struct TestResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub json: serde_json::Value,
}

/// 요청 하나를 보내고 JSON 본문까지 읽는다(본문이 비었으면 `Null`).
pub async fn send(app: &Router, req: Request<Body>) -> TestResponse {
    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let headers = res.headers().clone();
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let json = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    TestResponse {
        status,
        headers,
        json,
    }
}
