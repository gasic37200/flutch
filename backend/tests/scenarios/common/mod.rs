//! 테스트용 앱 생성. DB는 `#[sqlx::test]`가 준 진짜 Postgres, 외부 연동(LiveKit·토스)만 fake.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use axum::Router;
use axum::body::Body;
use axum::http::{HeaderMap, Request, StatusCode};
use flutch::config::{AuthConfig, BroadcastConfig, Config, LiveConfig, LiveKitConfig};
use flutch::livekit::{LiveKitApi, LiveKitError};
use flutch::toss::TossApi;
use flutch::{AppState, realtime};
use sqlx::PgPool;
use tower::ServiceExt;

pub const ORIGIN: &str = "http://localhost:5173";
/// 테스트 앱의 시드 계정(streamer1·streamer2·viewer) 비밀번호.
pub const SEED_PASSWORD: &str = "seedpass123";
/// `signup` 헬퍼가 쓰는 비밀번호.
pub const PASSWORD: &str = "testpass123";

/// trait에 메서드가 생기면 여기서 호출 기록·응답을 조작해 웹훅 반복·실패를 재현한다.
#[derive(Default)]
pub struct FakeLiveKit {
    /// `delete_room`으로 닫은 방 이름(호출 순서대로).
    pub deleted_rooms: Mutex<Vec<String>>,
    /// `remove_participant`로 내보낸 (방, 신원).
    pub removed_participants: Mutex<Vec<(String, String)>>,
    /// `list_participants`가 돌려줄 방별 참가자 신원(없는 방은 빈 목록).
    pub participants: Mutex<HashMap<String, Vec<String>>>,
    /// true면 `list_participants`가 실패한다.
    pub list_fails: AtomicBool,
}

#[async_trait]
impl LiveKitApi for FakeLiveKit {
    async fn delete_room(&self, room: &str) -> Result<(), LiveKitError> {
        self.deleted_rooms.lock().unwrap().push(room.to_owned());
        Ok(())
    }

    async fn remove_participant(&self, room: &str, identity: &str) -> Result<(), LiveKitError> {
        self.removed_participants
            .lock()
            .unwrap()
            .push((room.to_owned(), identity.to_owned()));
        Ok(())
    }

    async fn list_participants(&self, room: &str) -> Result<Vec<String>, LiveKitError> {
        if self.list_fails.load(Ordering::SeqCst) {
            return Err(LiveKitError::Status(503));
        }
        Ok(self
            .participants
            .lock()
            .unwrap()
            .get(room)
            .cloned()
            .unwrap_or_default())
    }
}

pub struct FakeToss;
impl TossApi for FakeToss {}

pub fn test_app(db: PgPool) -> Router {
    flutch::app(test_state(db, Arc::new(FakeLiveKit::default())))
}

/// 앱 상태. 호출 기록을 보려면 `FakeLiveKit`을 직접 만들어 넘기고 같은 `Arc`를 들고 있는다.
pub fn test_state(db: PgPool, livekit: Arc<FakeLiveKit>) -> Arc<AppState> {
    let config = Config {
        backend_port: 0,
        database_url: String::new(),
        allowed_origins: vec![ORIGIN.to_string()],
        livekit: LiveKitConfig {
            url: "ws://livekit.test".into(),
            api_key: "devkey".into(),
            api_secret: "secret".into(),
        },
        auth: AuthConfig {
            seed_password: Some(SEED_PASSWORD.into()),
        },
        live: LiveConfig {
            grace_secs: 60,
            max_duration_secs: 3 * 60 * 60,
        },
        broadcast: BroadcastConfig {
            thumb_dir: std::env::temp_dir().join("flutch-test-thumbs"),
        },
    };
    Arc::new(AppState {
        config,
        db,
        livekit,
        toss: Arc::new(FakeToss),
        realtime: realtime::Hub::default(),
        live: Default::default(),
    })
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

/// JSON 요청. 변경 요청이 Origin 검사를 통과하도록 `Origin`을 붙이고, `cookie`가 있으면 세션 쿠키로 보낸다.
pub fn json_req(
    method: &str,
    path: &str,
    cookie: Option<&str>,
    body: serde_json::Value,
) -> Request<Body> {
    let mut req = Request::builder()
        .method(method)
        .uri(path)
        .header("origin", ORIGIN)
        .header("content-type", "application/json");
    if let Some(c) = cookie {
        req = req.header("cookie", c);
    }
    req.body(Body::from(body.to_string())).unwrap()
}

/// 본문 없는 요청(GET·로그아웃 등).
pub fn empty_req(method: &str, path: &str, cookie: Option<&str>) -> Request<Body> {
    let mut req = Request::builder()
        .method(method)
        .uri(path)
        .header("origin", ORIGIN);
    if let Some(c) = cookie {
        req = req.header("cookie", c);
    }
    req.body(Body::empty()).unwrap()
}

/// 응답의 `Set-Cookie`에서 `flutch_session=<토큰>` 부분만 꺼낸다(다음 요청의 `cookie` 헤더로 쓴다).
pub fn session_cookie(res: &TestResponse) -> String {
    let set = res.headers["set-cookie"].to_str().unwrap();
    set.split(';').next().unwrap().to_string()
}

/// 새 사용자로 가입하고 세션 쿠키를 돌려준다(비밀번호 `PASSWORD`). 가입 IP 제한을 피하려고 아이디마다 XFF를 다르게 준다.
pub async fn signup(app: &Router, login_id: &str, nickname: &str) -> String {
    let mut req = json_req(
        "POST",
        "/api/v1/auth/signup",
        None,
        serde_json::json!({
            "loginId": login_id, "password": PASSWORD, "nickname": nickname,
            "agreeTerms": true, "agreePrivacy": true, "over14": true,
        }),
    );
    req.headers_mut().insert(
        "x-forwarded-for",
        format!("signup-{login_id}").parse().unwrap(),
    );
    let res = send(app, req).await;
    assert_eq!(
        res.status,
        StatusCode::CREATED,
        "signup {login_id}: {}",
        res.json
    );
    session_cookie(&res)
}

/// 시드 계정(`streamer1`·`streamer2`·`viewer`)으로 테스트 로그인하고 `(세션 쿠키, 사용자 id)`를 돌려준다.
pub async fn login_as(app: &Router, account: &str) -> (String, i64) {
    let res = send(
        app,
        json_req(
            "POST",
            "/api/v1/auth/test-login",
            None,
            serde_json::json!({ "account": account }),
        ),
    )
    .await;
    assert_eq!(
        res.status,
        StatusCode::OK,
        "test-login {account}: {}",
        res.json
    );
    (session_cookie(&res), res.json["id"].as_i64().unwrap())
}
