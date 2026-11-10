//! 테스트용 앱 생성. DB는 `#[sqlx::test]`가 준 진짜 Postgres, 외부 연동(LiveKit·토스)만 fake.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use axum::Router;
use axum::body::Body;
use axum::http::{HeaderMap, Request, StatusCode};
use flutch::config::{
    AuthConfig, BroadcastConfig, Config, KakaoConfig, LiveConfig, LiveKitConfig, RecordingConfig,
    S3Config, TossConfig,
};
use flutch::kakao::{KakaoApi, KakaoError, KakaoProfile};
use flutch::livekit::{LiveKitApi, LiveKitError, Participant};
use flutch::recording::{ObjectStore, StorageError};
use flutch::toss::{TossApi, TossError, TossPayment};
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
    /// `list_participants`가 돌려줄 방별 참가자(없는 방은 빈 목록).
    pub participants: Mutex<HashMap<String, Vec<Participant>>>,
    /// true면 `list_participants`가 실패한다.
    pub list_fails: AtomicBool,
    /// `start_recording`으로 녹화를 시작한 (방, 객체 키).
    pub started_recordings: Mutex<Vec<(String, String)>>,
    /// `stop_recording`으로 멈춘 egress id.
    pub stopped_recordings: Mutex<Vec<String>>,
    /// true면 `start_recording`이 실패한다.
    pub recording_start_fails: AtomicBool,
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

    async fn list_participants(&self, room: &str) -> Result<Vec<Participant>, LiveKitError> {
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

    async fn start_recording(
        &self,
        room: &str,
        _s3: &S3Config,
        key: &str,
    ) -> Result<String, LiveKitError> {
        if self.recording_start_fails.load(Ordering::SeqCst) {
            return Err(LiveKitError::Status(503));
        }
        let mut started = self.started_recordings.lock().unwrap();
        started.push((room.to_owned(), key.to_owned()));
        Ok(format!("EG_{}", started.len()))
    }

    async fn stop_recording(&self, egress_id: &str) -> Result<(), LiveKitError> {
        self.stopped_recordings
            .lock()
            .unwrap()
            .push(egress_id.to_owned());
        Ok(())
    }
}

/// 녹화 저장소. 지운 객체 키를 기록하고, `delete_fails`면 실패한다.
#[derive(Default)]
pub struct FakeStore {
    pub deleted: Mutex<Vec<String>>,
    pub delete_fails: AtomicBool,
}

#[async_trait]
impl ObjectStore for FakeStore {
    async fn delete(&self, key: &str) -> Result<(), StorageError> {
        if self.delete_fails.load(Ordering::SeqCst) {
            return Err(StorageError::Status(503));
        }
        self.deleted.lock().unwrap().push(key.to_owned());
        Ok(())
    }
}

/// 다음 승인 호출 한 번의 결과.
pub enum FakeConfirm {
    /// 4xx 거절(카드 거절)
    Reject,
    /// 통신 실패(승인 안 됨)
    Unavailable,
    /// 토스는 승인했는데 응답을 못 받음(승인 기록은 남아 조회·웹훅에 나온다)
    ApproveThenUnavailable,
}

/// 실제 토스처럼 승인된 결제를 기억하고, 같은 paymentKey 재승인은 `ALREADY_PROCESSED_PAYMENT`로 거절한다.
#[derive(Default)]
pub struct FakeToss {
    /// 승인된 결제(paymentKey → 결제). 조회 API가 이것을 돌려준다.
    pub payments: Mutex<HashMap<String, TossPayment>>,
    /// 승인 API에 온 paymentKey(호출 순서대로)
    pub confirm_calls: Mutex<Vec<String>>,
    pub next_confirm: Mutex<Option<FakeConfirm>>,
    /// true면 조회 API가 통신 실패
    pub lookup_fails: AtomicBool,
    /// 승인 응답 지연(ms) — 동시 요청이 확실히 겹치게 한다
    pub confirm_delay_ms: AtomicU64,
}

impl FakeToss {
    /// 그 paymentKey의 결제 객체 `secret`(웹훅 본문에 넣을 값)
    pub fn secret(key: &str) -> String {
        format!("secret-{key}")
    }
}

#[async_trait]
impl TossApi for FakeToss {
    async fn confirm(
        &self,
        payment_key: &str,
        order_id: &str,
        amount: i64,
    ) -> Result<TossPayment, TossError> {
        self.confirm_calls
            .lock()
            .unwrap()
            .push(payment_key.to_owned());
        let delay = self.confirm_delay_ms.load(Ordering::SeqCst);
        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
        let next = self.next_confirm.lock().unwrap().take();
        let unavailable = || TossError::Unavailable("fake".into());
        match next {
            Some(FakeConfirm::Reject) => {
                return Err(TossError::Rejected {
                    code: "REJECT_CARD_PAYMENT".into(),
                    message: "한도초과 혹은 잔액부족".into(),
                });
            }
            Some(FakeConfirm::Unavailable) => return Err(unavailable()),
            _ => {}
        }
        let mut payments = self.payments.lock().unwrap();
        if payments.contains_key(payment_key) {
            return Err(TossError::Rejected {
                code: "ALREADY_PROCESSED_PAYMENT".into(),
                message: "이미 처리된 결제 입니다.".into(),
            });
        }
        let p = TossPayment {
            payment_key: payment_key.to_owned(),
            order_id: order_id.to_owned(),
            status: "DONE".into(),
            total_amount: amount,
            secret: Some(Self::secret(payment_key)),
        };
        payments.insert(payment_key.to_owned(), p.clone());
        match next {
            Some(FakeConfirm::ApproveThenUnavailable) => Err(unavailable()),
            _ => Ok(p),
        }
    }

    async fn payment(&self, payment_key: &str) -> Result<TossPayment, TossError> {
        if self.lookup_fails.load(Ordering::SeqCst) {
            return Err(TossError::Unavailable("fake".into()));
        }
        self.payments
            .lock()
            .unwrap()
            .get(payment_key)
            .cloned()
            .ok_or(TossError::Rejected {
                code: "NOT_FOUND_PAYMENT".into(),
                message: "존재하지 않는 결제 정보 입니다.".into(),
            })
    }
}

pub fn test_app(db: PgPool) -> Router {
    flutch::app(test_state(db, Arc::new(FakeLiveKit::default())))
}

/// 앱 상태. 호출 기록을 보려면 `FakeLiveKit`을 직접 만들어 넘기고 같은 `Arc`를 들고 있는다.
pub fn test_state(db: PgPool, livekit: Arc<FakeLiveKit>) -> Arc<AppState> {
    test_state_with(db, livekit, Arc::new(FakeStore::default()), None)
}

/// 녹화 설정을 켠 앱 상태. 녹화 시나리오가 쓴다(`recording`이 `None`이면 녹화 꺼짐).
pub fn test_state_with(
    db: PgPool,
    livekit: Arc<FakeLiveKit>,
    storage: Arc<FakeStore>,
    recording: Option<RecordingConfig>,
) -> Arc<AppState> {
    build_state(
        db,
        livekit,
        storage,
        recording,
        Arc::new(FakeToss::default()),
        Arc::new(FakeKakao::default()),
    )
}

/// 카카오 호출을 조작·확인하는 시나리오용. `profile`이 `None`이면 토큰 교환 실패.
#[derive(Default)]
pub struct FakeKakao {
    pub profile: Mutex<Option<KakaoProfile>>,
    /// `profile`에 넘어온 인가 코드(호출 순서대로)
    pub codes: Mutex<Vec<String>>,
    /// 연결을 끊은 카카오 id(호출 순서대로)
    pub unlinked: Mutex<Vec<i64>>,
}

impl FakeKakao {
    pub fn set_profile(&self, id: i64, nickname: Option<&str>) {
        *self.profile.lock().unwrap() = Some(KakaoProfile {
            id,
            nickname: nickname.map(str::to_owned),
        });
    }
}

#[async_trait]
impl KakaoApi for FakeKakao {
    async fn profile(&self, code: &str) -> Result<KakaoProfile, KakaoError> {
        self.codes.lock().unwrap().push(code.to_owned());
        self.profile
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| KakaoError("fake".into()))
    }

    async fn unlink(&self, kakao_id: i64) -> Result<(), KakaoError> {
        self.unlinked.lock().unwrap().push(kakao_id);
        Ok(())
    }
}

/// 카카오 로그인 시나리오용(같은 `Arc<FakeKakao>`를 들고 있는다).
pub fn test_app_with_kakao(db: PgPool, kakao: Arc<FakeKakao>) -> Router {
    flutch::app(build_state(
        db,
        Arc::new(FakeLiveKit::default()),
        Arc::new(FakeStore::default()),
        None,
        Arc::new(FakeToss::default()),
        kakao,
    ))
}

/// 토스 호출을 조작·확인하는 시나리오용(같은 `Arc<FakeToss>`를 들고 있는다).
pub fn test_state_with_toss(
    db: PgPool,
    livekit: Arc<FakeLiveKit>,
    toss: Arc<FakeToss>,
) -> Arc<AppState> {
    build_state(
        db,
        livekit,
        Arc::new(FakeStore::default()),
        None,
        toss,
        Arc::new(FakeKakao::default()),
    )
}

fn build_state(
    db: PgPool,
    livekit: Arc<FakeLiveKit>,
    storage: Arc<FakeStore>,
    recording: Option<RecordingConfig>,
    toss: Arc<FakeToss>,
    kakao: Arc<FakeKakao>,
) -> Arc<AppState> {
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
            kakao: Some(KakaoConfig {
                rest_key: "kakao-rest".into(),
                client_secret: Some("kakao-secret".into()),
                redirect_uri: "https://flutch.test/api/v1/auth/kakao/callback".into(),
                admin_key: Some("kakao-admin".into()),
            }),
        },
        live: LiveConfig {
            grace_secs: 60,
            max_duration_secs: 3 * 60 * 60,
        },
        broadcast: BroadcastConfig {
            thumb_dir: std::env::temp_dir().join("flutch-test-thumbs"),
        },
        recording: recording.unwrap_or(RecordingConfig {
            s3: None,
            public_base_url: String::new(),
            retention_days: 30,
            max_total_bytes: 16 * 1024 * 1024 * 1024,
        }),
        toss: TossConfig {
            client_key: Some("test_gck_fake".into()),
            secret_key: None,
        },
        clutch: Default::default(),
    };
    Arc::new(AppState {
        config,
        db,
        livekit,
        toss,
        kakao,
        storage,
        realtime: realtime::Hub::default(),
        live: Default::default(),
        clutch: Default::default(),
    })
}

/// 녹화가 켜진 테스트 설정(가짜 저장소, 공개 URL `https://cdn.test`).
pub fn recording_config() -> RecordingConfig {
    RecordingConfig {
        s3: Some(S3Config {
            endpoint: "https://s3.test".into(),
            bucket: "rec".into(),
            region: "test".into(),
            access_key: "ak".into(),
            secret_key: "sk".into(),
        }),
        public_base_url: "https://cdn.test/rec".into(),
        retention_days: 30,
        max_total_bytes: 16 * 1024 * 1024 * 1024,
    }
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
