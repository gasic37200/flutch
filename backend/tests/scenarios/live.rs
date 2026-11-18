//! 송출·시청 시나리오 (features/03, api.md §3-1).

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use flutch::AppState;
use flutch::broadcast::{EndReason, end_live};
use flutch::live::{check_limits, sync};
use flutch::livekit::{Participant, Permission};
use hmac::{Hmac, KeyInit, Mac};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::PgPool;

use crate::common::{
    FakeLiveKit, ORIGIN, TestResponse, empty_req, json_req, login_as, send, test_state,
};
use crate::realtime::{connect, next_event};

const B: &str = "/api/v1/broadcasts";
const TAB_A: &str = "tab-aaaaaaaa";
const TAB_B: &str = "tab-bbbbbbbb";
const TAB_C: &str = "tab-cccccccc";

fn setup(db: PgPool) -> (Router, Arc<FakeLiveKit>) {
    let (app, lk, _) = setup_state(db);
    (app, lk)
}

/// 주기 작업(`check_limits`·`sync`)을 직접 부르려고 상태도 돌려준다.
fn setup_state(db: PgPool) -> (Router, Arc<FakeLiveKit>, Arc<AppState>) {
    let lk = Arc::new(FakeLiveKit::default());
    let state = test_state(db, lk.clone());
    (flutch::app(state.clone()), lk, state)
}

/// 이벤트를 보려고 실제 서버도 띄운다(WebSocket).
async fn setup_server(db: PgPool) -> (Router, Arc<FakeLiveKit>, Arc<AppState>, SocketAddr) {
    let (app, lk, state) = setup_state(db);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = flutch::app(state.clone());
    tokio::spawn(async move { axum::serve(listener, server).await.unwrap() });
    (app, lk, state, addr)
}

fn hs256(secret: &str, claims: &Value) -> String {
    let header = URL_SAFE_NO_PAD.encode(r#"{"alg":"HS256","typ":"JWT"}"#);
    let payload = URL_SAFE_NO_PAD.encode(claims.to_string());
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(format!("{header}.{payload}").as_bytes());
    let sig = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
    format!("{header}.{payload}.{sig}")
}

/// LiveKit 웹훅과 같은 방식의 서명: `sha256` = 본문 SHA-256의 표준 base64, 만료는 지금 + `exp_in`초.
fn webhook_token(secret: &str, iss: &str, body: &str, exp_in: i64) -> String {
    let now = chrono::Utc::now().timestamp();
    let hash = STANDARD.encode(Sha256::digest(body.as_bytes()));
    hs256(
        secret,
        &json!({ "iss": iss, "nbf": now, "exp": now + exp_in, "sha256": hash, "video": {} }),
    )
}

async fn webhook_with(app: &Router, token: Option<&str>, body: &str) -> TestResponse {
    let mut req = Request::builder()
        .method("POST")
        .uri("/api/v1/livekit/webhook")
        .header("content-type", "application/webhook+json");
    if let Some(t) = token {
        req = req.header("authorization", t);
    }
    send(app, req.body(Body::from(body.to_owned())).unwrap()).await
}

/// 올바르게 서명한 웹훅. 200이어야 한다.
pub async fn webhook(app: &Router, ev: Value) {
    let body = ev.to_string();
    let token = webhook_token("secret", "devkey", &body, 300);
    let res = webhook_with(app, Some(&token), &body).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
}

/// LiveKit `WebhookEvent` 형식(protojson, false 값은 생략).
pub fn participant(event: &str, id: i64, identity: &str, can_publish: bool) -> Value {
    let permission = if can_publish {
        json!({ "canSubscribe": true, "canPublish": true })
    } else {
        json!({ "canSubscribe": true })
    };
    json!({
        "event": event, "id": "EV_test", "createdAt": "1762650000",
        "room": { "sid": "RM_test", "name": format!("broadcast-{id}") },
        "participant": { "sid": "PA_test", "identity": identity, "state": "ACTIVE", "permission": permission },
    })
}

/// `list_participants`가 돌려줄 참가자.
fn member(identity: &str, can_publish: bool) -> Participant {
    Participant {
        identity: identity.to_owned(),
        sid: format!("PA_{identity}"),
        permission: Permission { can_publish },
    }
}

fn room_finished(id: i64) -> Value {
    json!({ "event": "room_finished", "room": { "sid": "RM_test", "name": format!("broadcast-{id}") } })
}

async fn detail(app: &Router, id: i64) -> Value {
    send(app, empty_req("GET", &format!("{B}/{id}"), None))
        .await
        .json
}

/// 시간 조작: 유예 시작을 `secs`초 전으로.
async fn backdate_disconnect(db: &PgPool, id: i64, secs: i64) {
    sqlx::query(
        "UPDATE broadcasts SET disconnected_at = now() - $2::bigint * interval '1 second' WHERE id = $1",
    )
    .bind(id)
    .bind(secs)
    .execute(db)
    .await
    .unwrap();
}

pub async fn create(app: &Router, cookie: &str) -> i64 {
    let body = json!({ "title": "송출 테스트", "category": "game" });
    let res = send(app, json_req("POST", B, Some(cookie), body)).await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.json);
    res.json["id"].as_i64().unwrap()
}

pub async fn go_live(app: &Router, cookie: Option<&str>, id: i64, body: Value) -> TestResponse {
    send(
        app,
        json_req("POST", &format!("{B}/{id}/live"), cookie, body),
    )
    .await
}

async fn viewer_token(app: &Router, cookie: Option<&str>, id: i64, body: Value) -> TestResponse {
    send(
        app,
        json_req("POST", &format!("{B}/{id}/viewer-token"), cookie, body),
    )
    .await
}

fn code(res: &TestResponse) -> &str {
    res.json["error"]["code"].as_str().unwrap()
}

/// 서명(테스트 설정의 secret)을 확인하고 claims를 꺼낸다.
fn claims(token: &Value) -> Value {
    let parts: Vec<_> = token.as_str().unwrap().split('.').collect();
    let mut mac = Hmac::<Sha256>::new_from_slice(b"secret").unwrap();
    mac.update(format!("{}.{}", parts[0], parts[1]).as_bytes());
    assert_eq!(
        parts[2],
        URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes())
    );
    serde_json::from_slice(&URL_SAFE_NO_PAD.decode(parts[1]).unwrap()).unwrap()
}

async fn publisher_identity(db: &PgPool, id: i64) -> Option<String> {
    sqlx::query_scalar("SELECT publisher_identity FROM broadcasts WHERE id = $1")
        .bind(id)
        .fetch_one(db)
        .await
        .unwrap()
}

#[sqlx::test]
async fn live_start_owner_only(db: PgPool) {
    let (app, _lk) = setup(db.clone());
    let (owner, uid) = login_as(&app, "streamer1").await;
    let (other, _) = login_as(&app, "streamer2").await;
    let (viewer, vid) = login_as(&app, "viewer").await;
    let id = create(&app, &owner).await;
    let tab = json!({ "tabId": TAB_A });

    assert_eq!(
        go_live(&app, None, id, tab.clone()).await.status,
        StatusCode::UNAUTHORIZED
    );
    let res = go_live(&app, Some(&other), id, tab.clone()).await;
    assert_eq!(res.status, StatusCode::FORBIDDEN);
    assert_eq!(code(&res), "BROADCAST_NOT_OWNER");
    assert_eq!(
        go_live(&app, Some(&owner), 999_999, tab.clone())
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    let res = go_live(&app, Some(&owner), id, json!({ "tabId": "short" })).await;
    assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(res.json["error"]["fields"]["tabId"].is_string());
    // 실패한 요청은 상태를 바꾸지 않는다
    let res = send(&app, empty_req("GET", &format!("{B}/{id}"), None)).await;
    assert_eq!(res.json["status"], "READY");

    let res = go_live(&app, Some(&owner), id, tab.clone()).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    assert_eq!(res.json["broadcast"]["status"], "LIVE");
    assert_eq!(res.json["broadcast"]["viewer"]["isOwner"], true);
    let lk = &res.json["livekit"];
    let identity = format!("u:{uid}:{TAB_A}");
    let room = format!("broadcast-{id}");
    assert_eq!(lk["identity"], identity.as_str());
    assert_eq!(lk["room"], room.as_str());
    assert_eq!(lk["url"], "ws://livekit.test");
    let c = claims(&lk["token"]);
    assert_eq!(c["iss"], "devkey");
    assert_eq!(c["sub"], identity.as_str());
    assert_eq!(
        c["exp"].as_i64().unwrap() - c["nbf"].as_i64().unwrap(),
        4 * 3600
    );
    let v = &c["video"];
    assert_eq!(v["room"], room.as_str());
    assert_eq!(v["roomJoin"], true);
    assert_eq!(v["canPublish"], true);
    assert_eq!(v["canSubscribe"], true);
    assert_eq!(v["canPublishData"], false);
    assert_eq!(
        v["canPublishSources"],
        json!(["camera", "microphone", "screen_share", "screen_share_audio"])
    );
    assert!(v.get("hidden").is_none());
    assert_eq!(
        publisher_identity(&db, id).await.as_deref(),
        Some(identity.as_str())
    );

    // 시청 토큰은 구독만
    let res = viewer_token(&app, Some(&viewer), id, json!({ "tabId": TAB_B })).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    assert_eq!(res.json["identity"], format!("u:{vid}:{TAB_B}").as_str());
    let v = claims(&res.json["token"])["video"].clone();
    assert_eq!(v["roomJoin"], true);
    assert_eq!(v["canSubscribe"], true);
    assert_eq!(v["canPublish"], false);
    assert_eq!(v["canPublishData"], false);
    assert!(v.get("canPublishSources").is_none());
    // 시청자 신원은 다른 참가자에게 보이지 않는다
    assert_eq!(v["hidden"], true);

    // 동시 LIVE 1개
    let second = create(&app, &owner).await;
    let res = go_live(&app, Some(&owner), second, json!({ "tabId": TAB_B })).await;
    assert_eq!(res.status, StatusCode::CONFLICT);
    assert_eq!(code(&res), "BROADCAST_LIVE_LIMIT");
}

#[sqlx::test]
async fn live_end_closes_room(db: PgPool) {
    let (app, lk) = setup(db);
    let (owner, _) = login_as(&app, "streamer1").await;
    let (other, _) = login_as(&app, "streamer2").await;
    let id = create(&app, &owner).await;
    let path = format!("{B}/{id}/live");

    // READY는 종료할 수 없다
    let res = send(&app, empty_req("DELETE", &path, Some(&owner))).await;
    assert_eq!(res.status, StatusCode::CONFLICT);
    assert_eq!(code(&res), "BROADCAST_INVALID_TRANSITION");

    let res = go_live(&app, Some(&owner), id, json!({ "tabId": TAB_A })).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);

    assert_eq!(
        send(&app, empty_req("DELETE", &path, None)).await.status,
        StatusCode::UNAUTHORIZED
    );
    let res = send(&app, empty_req("DELETE", &path, Some(&other))).await;
    assert_eq!(res.status, StatusCode::FORBIDDEN);
    assert!(lk.deleted_rooms.lock().unwrap().is_empty());

    let res = send(&app, empty_req("DELETE", &path, Some(&owner))).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    assert_eq!(res.json["status"], "ENDED");
    assert!(res.json["endedAt"].is_string());
    assert_eq!(
        *lk.deleted_rooms.lock().unwrap(),
        vec![format!("broadcast-{id}")]
    );

    let res = send(&app, empty_req("DELETE", &path, Some(&owner))).await;
    assert_eq!(res.status, StatusCode::CONFLICT);
    assert_eq!(code(&res), "BROADCAST_INVALID_TRANSITION");
    // ENDED는 다시 송출할 수 없다
    let res = go_live(&app, Some(&owner), id, json!({ "tabId": TAB_A })).await;
    assert_eq!(res.status, StatusCode::CONFLICT);
    assert_eq!(code(&res), "BROADCAST_INVALID_TRANSITION");
    assert_eq!(lk.deleted_rooms.lock().unwrap().len(), 1);
}

#[sqlx::test]
async fn live_takeover_other_device(db: PgPool) {
    let (app, lk) = setup(db.clone());
    let (owner, uid) = login_as(&app, "streamer1").await;
    let id = create(&app, &owner).await;
    let (a, b) = (format!("u:{uid}:{TAB_A}"), format!("u:{uid}:{TAB_B}"));
    let room = format!("broadcast-{id}");

    assert_eq!(
        go_live(&app, Some(&owner), id, json!({ "tabId": TAB_A }))
            .await
            .status,
        StatusCode::OK
    );
    // 접속 웹훅 전에는 유예 중(접속하지 못하면 유예 만료로 끝난다)
    assert_eq!(detail(&app, id).await["reconnecting"], true);
    webhook(&app, participant("participant_joined", id, &a, true)).await;
    // 같은 탭(새로고침)은 새 토큰만
    let res = go_live(&app, Some(&owner), id, json!({ "tabId": TAB_A })).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.json["livekit"]["identity"], a.as_str());
    assert!(lk.removed_participants.lock().unwrap().is_empty());

    // 다른 기기는 확인 필요
    let res = go_live(&app, Some(&owner), id, json!({ "tabId": TAB_B })).await;
    assert_eq!(res.status, StatusCode::CONFLICT);
    assert_eq!(code(&res), "LIVE_PUBLISHER_ACTIVE");
    assert_eq!(
        publisher_identity(&db, id).await.as_deref(),
        Some(a.as_str())
    );

    // 전환: 기존 기기를 내보내고 새 기기가 송출자
    let res = go_live(
        &app,
        Some(&owner),
        id,
        json!({ "tabId": TAB_B, "takeover": true }),
    )
    .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    assert_eq!(res.json["livekit"]["identity"], b.as_str());
    assert_eq!(res.json["broadcast"]["status"], "LIVE");
    assert_eq!(
        *lk.removed_participants.lock().unwrap(),
        vec![(room.clone(), a.clone())]
    );
    assert_eq!(
        publisher_identity(&db, id).await.as_deref(),
        Some(b.as_str())
    );

    // 밀려난 기기가 옛 송출 토큰으로 LiveKit에 다시 들어오면 웹훅이 내보낸다
    webhook(&app, participant("participant_joined", id, &a, true)).await;
    assert_eq!(
        *lk.removed_participants.lock().unwrap(),
        vec![(room.clone(), a.clone()), (room.clone(), a.clone())]
    );
    // 새 송출자와 소유자의 시청 탭(구독 전용)은 그대로
    webhook(&app, participant("participant_joined", id, &b, true)).await;
    let owner_viewer = format!("u:{uid}:{TAB_C}");
    webhook(
        &app,
        participant("participant_joined", id, &owner_viewer, false),
    )
    .await;
    assert_eq!(lk.removed_participants.lock().unwrap().len(), 2);

    // 밀려난 기기가 그냥 다시 시작하면 다시 확인
    let res = go_live(&app, Some(&owner), id, json!({ "tabId": TAB_A })).await;
    assert_eq!(code(&res), "LIVE_PUBLISHER_ACTIVE");

    // 송출자가 끊겨(퇴장 웹훅) 유예 중이면 확인 없이 이어서 송출
    webhook(&app, participant("participant_left", id, &b, true)).await;
    assert_eq!(detail(&app, id).await["reconnecting"], true);
    let res = go_live(&app, Some(&owner), id, json!({ "tabId": TAB_A })).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    assert_eq!(
        publisher_identity(&db, id).await.as_deref(),
        Some(a.as_str())
    );
    assert_eq!(
        lk.removed_participants.lock().unwrap().last(),
        Some(&(room, b))
    );
    // 새 기기의 참가 웹훅이 오면 유예 해제
    assert_eq!(detail(&app, id).await["reconnecting"], true);
    webhook(&app, participant("participant_joined", id, &a, true)).await;
    assert_eq!(detail(&app, id).await["reconnecting"], false);
}

#[sqlx::test]
async fn live_viewer_token_rules(db: PgPool) {
    let (app, _lk) = setup(db);
    let (owner, _) = login_as(&app, "streamer1").await;
    let (viewer, vid) = login_as(&app, "viewer").await;
    let id = create(&app, &owner).await;
    let guest = json!({ "tabId": TAB_A, "guestId": "guest-12345678" });

    assert_eq!(
        viewer_token(&app, None, 999_999, guest.clone())
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    // LIVE 전
    let res = viewer_token(&app, None, id, guest.clone()).await;
    assert_eq!(res.status, StatusCode::CONFLICT);
    assert_eq!(code(&res), "BROADCAST_NOT_LIVE");

    go_live(&app, Some(&owner), id, json!({ "tabId": TAB_B })).await;

    // 비로그인은 guestId 필수·형식 검사
    let res = viewer_token(&app, None, id, json!({ "tabId": TAB_A })).await;
    assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(res.json["error"]["fields"]["guestId"].is_string());
    let res = viewer_token(
        &app,
        None,
        id,
        json!({ "tabId": TAB_A, "guestId": "bad id!!" }),
    )
    .await;
    assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY);
    let res = viewer_token(
        &app,
        None,
        id,
        json!({ "tabId": "x".repeat(65), "guestId": "guest-12345678" }),
    )
    .await;
    assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(res.json["error"]["fields"]["tabId"].is_string());

    let res = viewer_token(&app, None, id, guest.clone()).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    assert_eq!(
        res.json["identity"],
        format!("g:guest-12345678:{TAB_A}").as_str()
    );
    assert_eq!(res.json["room"], format!("broadcast-{id}").as_str());
    assert_eq!(claims(&res.json["token"])["video"]["canPublish"], false);

    // 로그인 사용자의 guestId는 무시
    let res = viewer_token(
        &app,
        Some(&viewer),
        id,
        json!({ "tabId": TAB_A, "guestId": "bad id!!" }),
    )
    .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    assert_eq!(res.json["identity"], format!("u:{vid}:{TAB_A}").as_str());

    // 종료 후
    send(
        &app,
        empty_req("DELETE", &format!("{B}/{id}/live"), Some(&owner)),
    )
    .await;
    let res = viewer_token(&app, None, id, guest).await;
    assert_eq!(code(&res), "BROADCAST_NOT_LIVE");
}

#[sqlx::test]
async fn live_webhook_signature_invalid_rejected(db: PgPool) {
    let (app, _lk) = setup(db);
    let (owner, uid) = login_as(&app, "streamer1").await;
    let id = create(&app, &owner).await;
    go_live(&app, Some(&owner), id, json!({ "tabId": TAB_A })).await;
    let a = format!("u:{uid}:{TAB_A}");
    webhook(&app, participant("participant_joined", id, &a, true)).await;
    let body = participant("participant_left", id, &a, true).to_string();

    let bad = [
        None,
        Some(webhook_token("wrong-secret", "devkey", &body, 300)),
        Some(webhook_token("secret", "other-key", &body, 300)),
        Some(webhook_token("secret", "devkey", "{}", 300)), // 다른 본문의 해시
        Some(webhook_token("secret", "devkey", &body, -300)), // 만료
        Some("not-a-jwt".to_owned()),
    ];
    for token in bad {
        let res = webhook_with(&app, token.as_deref(), &body).await;
        assert_eq!(res.status, StatusCode::UNAUTHORIZED, "{token:?}");
        assert_eq!(code(&res), "LIVE_WEBHOOK_INVALID_SIGNATURE");
    }
    // 거절된 웹훅은 상태를 바꾸지 않는다
    assert_eq!(detail(&app, id).await["reconnecting"], false);

    let token = webhook_token("secret", "devkey", &body, 300);
    let res = webhook_with(&app, Some(&token), &body).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    assert_eq!(detail(&app, id).await["reconnecting"], true);
    // 우리 방이 아니거나 모르는 이벤트는 200 무시
    webhook(
        &app,
        json!({ "event": "room_started", "room": { "name": "other" } }),
    )
    .await;
    webhook(
        &app,
        json!({ "event": "egress_ended", "room": { "name": format!("broadcast-{id}") } }),
    )
    .await;
}

#[sqlx::test]
async fn live_disconnect_grace_auto_end(db: PgPool) {
    let (app, lk, state, addr) = setup_server(db.clone()).await;
    let (owner, uid) = login_as(&app, "streamer1").await;
    let id = create(&app, &owner).await;
    go_live(&app, Some(&owner), id, json!({ "tabId": TAB_A })).await;
    let a = format!("u:{uid}:{TAB_A}");
    let room = format!("broadcast-{id}");
    let mut ws = connect(addr, id, Some(ORIGIN), None).await.unwrap();

    // 첫 접속이 시작 때의 유예를 푼다
    webhook(&app, participant("participant_joined", id, &a, true)).await;
    assert_eq!(next_event(&mut ws).await["type"], "broadcast.resumed");
    // 송출자 퇴장 → 유예 시작
    webhook(&app, participant("participant_left", id, &a, true)).await;
    let ev = next_event(&mut ws).await;
    assert_eq!(ev["type"], "broadcast.reconnecting");
    assert!(ev["data"]["since"].is_string());
    let d = detail(&app, id).await;
    assert_eq!(
        (d["status"].as_str(), d["reconnecting"].as_bool()),
        (Some("LIVE"), Some(true))
    );
    // 60초 전에는 유지
    check_limits(&state).await.unwrap();
    assert_eq!(detail(&app, id).await["status"], "LIVE");

    // 60초 안에 복귀
    webhook(&app, participant("participant_joined", id, &a, true)).await;
    assert_eq!(next_event(&mut ws).await["type"], "broadcast.resumed");
    assert_eq!(detail(&app, id).await["reconnecting"], false);
    // 만료 판정 직후 송출자가 돌아와 유예가 풀렸으면 끝내지 않는다
    end_live(&state, id, EndReason::Disconnect).await.unwrap();
    assert_eq!(detail(&app, id).await["status"], "LIVE");

    // 웹훅 순서가 뒤바뀌어 유예가 지났어도 송출자가 실제로 방에 있으면 유지
    webhook(&app, participant("participant_left", id, &a, true)).await;
    assert_eq!(next_event(&mut ws).await["type"], "broadcast.reconnecting");
    backdate_disconnect(&db, id, 61).await;
    lk.participants
        .lock()
        .unwrap()
        .insert(room.clone(), vec![member(&a, true)]);
    check_limits(&state).await.unwrap();
    assert_eq!(next_event(&mut ws).await["type"], "broadcast.resumed");
    assert_eq!(detail(&app, id).await["status"], "LIVE");
    assert!(lk.deleted_rooms.lock().unwrap().is_empty());

    // 60초 동안 복귀 없음 → 자동 종료 + 방 삭제
    webhook(&app, participant("participant_left", id, &a, true)).await;
    assert_eq!(next_event(&mut ws).await["type"], "broadcast.reconnecting");
    lk.participants.lock().unwrap().clear();
    backdate_disconnect(&db, id, 61).await;
    check_limits(&state).await.unwrap();
    let ev = next_event(&mut ws).await;
    assert_eq!(ev["type"], "broadcast.ended");
    assert_eq!(ev["data"]["reason"], "disconnect");
    let d = detail(&app, id).await;
    assert_eq!(
        (d["status"].as_str(), d["reconnecting"].as_bool()),
        (Some("ENDED"), Some(false))
    );
    assert_eq!(*lk.deleted_rooms.lock().unwrap(), vec![room]);

    // 종료 후 웹훅은 상태를 바꾸지 않는다
    webhook(&app, participant("participant_joined", id, &a, true)).await;
    webhook(&app, room_finished(id)).await;
    assert_eq!(detail(&app, id).await["status"], "ENDED");
}

#[sqlx::test]
async fn live_max_duration_auto_end(db: PgPool) {
    let (app, lk, state, addr) = setup_server(db.clone()).await;
    let (owner, _) = login_as(&app, "streamer1").await;
    let id = create(&app, &owner).await;
    go_live(&app, Some(&owner), id, json!({ "tabId": TAB_A })).await;
    let mut owner_ws = connect(addr, id, Some(ORIGIN), Some(&owner)).await.unwrap();
    let mut guest_ws = connect(addr, id, Some(ORIGIN), None).await.unwrap();
    let set_started = |ago: &'static str| {
        let db = db.clone();
        async move {
            sqlx::query("UPDATE broadcasts SET started_at = now() - $2::interval WHERE id = $1")
                .bind(id)
                .bind(ago)
                .execute(&db)
                .await
                .unwrap();
        }
    };

    // 10분 전 이전에는 경고 없음
    set_started("2 hours 49 minutes").await;
    check_limits(&state).await.unwrap();
    // 10분 전 → 소유자에게만 한 번
    set_started("2 hours 51 minutes").await;
    check_limits(&state).await.unwrap();
    check_limits(&state).await.unwrap();
    let ev = next_event(&mut owner_ws).await;
    assert_eq!(ev["type"], "broadcast.duration_warning");
    let started: chrono::DateTime<chrono::Utc> =
        sqlx::query_scalar("SELECT started_at FROM broadcasts WHERE id = $1")
            .bind(id)
            .fetch_one(&db)
            .await
            .unwrap();
    let ends_at: chrono::DateTime<chrono::Utc> =
        ev["data"]["endsAt"].as_str().unwrap().parse().unwrap();
    assert_eq!(ends_at, started + chrono::Duration::hours(3));
    assert_eq!(detail(&app, id).await["status"], "LIVE");

    // 3시간 초과 → 종료. 두 연결 모두 다음 이벤트가 종료다(경고 중복 없음, 시청자는 경고 없음)
    set_started("3 hours 1 second").await;
    check_limits(&state).await.unwrap();
    for ws in [&mut owner_ws, &mut guest_ws] {
        let ev = next_event(ws).await;
        assert_eq!(ev["type"], "broadcast.ended");
        assert_eq!(ev["data"]["reason"], "max_duration");
    }
    assert_eq!(detail(&app, id).await["status"], "ENDED");
    assert_eq!(
        *lk.deleted_rooms.lock().unwrap(),
        vec![format!("broadcast-{id}")]
    );
}

#[sqlx::test]
async fn live_viewer_count(db: PgPool) {
    let (app, _lk, _state, addr) = setup_server(db).await;
    let (owner, uid) = login_as(&app, "streamer1").await;
    let (other, _) = login_as(&app, "streamer2").await;
    let (_, vid) = login_as(&app, "viewer").await;
    let id = create(&app, &owner).await;
    go_live(&app, Some(&owner), id, json!({ "tabId": TAB_A })).await;
    let mut ws = connect(addr, id, Some(ORIGIN), None).await.unwrap();
    let join =
        |identity: String, publish: bool| participant("participant_joined", id, &identity, publish);
    let leave = |identity: String| participant("participant_left", id, &identity, false);

    // 송출자·소유자의 시청 탭·우리 토큰이 아닌 참가자(Egress 등)는 세지 않는다
    webhook(&app, join(format!("u:{uid}:{TAB_A}"), true)).await;
    assert_eq!(next_event(&mut ws).await["type"], "broadcast.resumed");
    webhook(&app, join(format!("u:{uid}:{TAB_C}"), false)).await;
    webhook(&app, join("EG_recorder".into(), false)).await;
    assert_eq!(detail(&app, id).await["viewerCount"], 0);

    webhook(&app, join(format!("u:{vid}:{TAB_A}"), false)).await;
    assert_eq!(
        next_event(&mut ws).await,
        json!({ "type": "viewer.count", "data": { "count": 1 } })
    );
    // 같은 사용자 다른 탭은 1명
    webhook(&app, join(format!("u:{vid}:{TAB_B}"), false)).await;
    webhook(&app, join(format!("g:guest-12345678:{TAB_A}"), false)).await;
    assert_eq!(next_event(&mut ws).await["data"]["count"], 2);
    webhook(&app, leave(format!("u:{vid}:{TAB_A}"))).await;
    webhook(&app, leave(format!("u:{vid}:{TAB_B}"))).await;
    assert_eq!(next_event(&mut ws).await["data"]["count"], 1);
    let d = detail(&app, id).await;
    assert_eq!(
        (d["viewerCount"].as_i64(), d["peakViewers"].as_i64()),
        (Some(1), Some(2))
    );

    // 목록은 시청자 수 순(더 최근 방송이라도 시청자가 적으면 뒤), 커서는 순위 위치
    let second = create(&app, &other).await;
    go_live(&app, Some(&other), second, json!({ "tabId": TAB_A })).await;
    let ids = |res: &TestResponse| -> Vec<i64> {
        res.json["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| b["id"].as_i64().unwrap())
            .collect()
    };
    let res = send(&app, empty_req("GET", B, None)).await;
    assert_eq!(ids(&res), vec![id, second]);
    assert_eq!(res.json["items"][0]["viewerCount"], 1);
    let res = send(&app, empty_req("GET", &format!("{B}?limit=1"), None)).await;
    assert_eq!(
        (ids(&res), res.json["nextCursor"].as_str()),
        (vec![id], Some("1"))
    );
    let res = send(
        &app,
        empty_req("GET", &format!("{B}?limit=1&cursor=1"), None),
    )
    .await;
    assert_eq!(
        (ids(&res), res.json["nextCursor"].is_null()),
        (vec![second], true)
    );

    // 같은 신원 재접속: 새 참가(새 sid)가 옛 퇴장보다 먼저 와도 수는 그대로
    let guest = format!("g:guest-12345678:{TAB_A}");
    let mut rejoin = join(guest.clone(), false);
    rejoin["participant"]["sid"] = json!("PA_new");
    webhook(&app, rejoin).await;
    webhook(&app, leave(guest)).await; // 옛 접속(PA_test)의 퇴장
    assert_eq!(detail(&app, id).await["viewerCount"], 1);

    // 방이 닫혀도(빈 방 정리) 바로 끝내지 않고 유예
    webhook(&app, room_finished(id)).await;
    assert_eq!(next_event(&mut ws).await["data"]["count"], 0);
    assert_eq!(next_event(&mut ws).await["type"], "broadcast.reconnecting");
    let d = detail(&app, id).await;
    assert_eq!(
        (
            d["status"].as_str(),
            d["reconnecting"].as_bool(),
            d["viewerCount"].as_i64()
        ),
        (Some("LIVE"), Some(true), Some(0))
    );
}

#[sqlx::test]
async fn live_restart_sync(db: PgPool) {
    let (app, lk, state) = setup_state(db.clone());
    let (s1, uid1) = login_as(&app, "streamer1").await;
    let (s2, uid2) = login_as(&app, "streamer2").await;
    let (_, vid) = login_as(&app, "viewer").await;
    let b1 = create(&app, &s1).await;
    let b2 = create(&app, &s2).await;
    go_live(&app, Some(&s1), b1, json!({ "tabId": TAB_A })).await;
    go_live(&app, Some(&s2), b2, json!({ "tabId": TAB_A })).await;
    let p2 = format!("u:{uid2}:{TAB_A}");
    webhook(&app, participant("participant_joined", b2, &p2, true)).await;
    // 재시작 전: b1 송출자가 끊겼다가 돌아왔는데 그 웹훅을 놓쳤다 / b2 송출자는 서버가 꺼진 사이 나갔다
    backdate_disconnect(&db, b1, 10).await;
    lk.participants.lock().unwrap().insert(
        format!("broadcast-{b1}"),
        vec![
            member(&format!("u:{uid1}:{TAB_A}"), true),
            member(&format!("u:{uid1}:{TAB_C}"), false),
            member(&format!("u:{vid}:{TAB_A}"), false),
            member(&format!("u:{vid}:{TAB_B}"), false),
            member(&format!("g:guest-12345678:{TAB_A}"), false),
        ],
    );

    // LiveKit 호출 실패 → DB는 그대로
    lk.list_fails.store(true, Ordering::SeqCst);
    assert!(sync(&state).await.is_err());
    assert_eq!(detail(&app, b1).await["reconnecting"], true);
    assert_eq!(detail(&app, b2).await["reconnecting"], false);

    lk.list_fails.store(false, Ordering::SeqCst);
    sync(&state).await.unwrap();
    let d1 = detail(&app, b1).await;
    assert_eq!(
        (
            d1["reconnecting"].as_bool(),
            d1["viewerCount"].as_i64(),
            d1["peakViewers"].as_i64()
        ),
        (Some(false), Some(2), Some(2))
    );
    let d2 = detail(&app, b2).await;
    assert_eq!(
        (d2["status"].as_str(), d2["reconnecting"].as_bool()),
        (Some("LIVE"), Some(true))
    );

    // 송출자가 없던 방송은 유예 흐름으로 자연 종료
    backdate_disconnect(&db, b2, 61).await;
    check_limits(&state).await.unwrap();
    assert_eq!(detail(&app, b2).await["status"], "ENDED");
    assert_eq!(detail(&app, b1).await["status"], "LIVE");
}

#[sqlx::test]
async fn live_publisher_never_joins_auto_end(db: PgPool) {
    let (app, lk, state, addr) = setup_server(db.clone()).await;
    let (owner, uid) = login_as(&app, "streamer1").await;
    let id = create(&app, &owner).await;
    let a = format!("u:{uid}:{TAB_A}");
    let room = format!("broadcast-{id}");
    let mut ws = connect(addr, id, Some(ORIGIN), None).await.unwrap();

    let res = go_live(&app, Some(&owner), id, json!({ "tabId": TAB_A })).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    assert_eq!(next_event(&mut ws).await["type"], "broadcast.updated");
    // 송출자가 아직 접속하지 않았다 → 유예 중
    assert_eq!(detail(&app, id).await["reconnecting"], true);
    check_limits(&state).await.unwrap();
    assert_eq!(detail(&app, id).await["status"], "LIVE");

    // 끝내 접속하지 못하고(연결 실패·탭 닫음, 웹훅 없음) 60초 → 자동 종료. 그 사이 다른 이벤트 없음
    backdate_disconnect(&db, id, 61).await;
    check_limits(&state).await.unwrap();
    let ev = next_event(&mut ws).await;
    assert_eq!(ev["type"], "broadcast.ended");
    assert_eq!(ev["data"]["reason"], "disconnect");
    assert_eq!(*lk.deleted_rooms.lock().unwrap(), vec![room.clone()]);

    // 끝난 방송의 옛 송출 토큰으로 들어오면 내보낸다(방이 다시 살아나지 않게). 시청자는 두고.
    webhook(&app, participant("participant_joined", id, &a, true)).await;
    let guest = format!("g:guest-12345678:{TAB_B}");
    webhook(&app, participant("participant_joined", id, &guest, false)).await;
    assert_eq!(*lk.removed_participants.lock().unwrap(), vec![(room, a)]);
    assert_eq!(detail(&app, id).await["status"], "ENDED");
}

#[sqlx::test]
async fn live_subscriber_same_identity_keeps_grace(db: PgPool) {
    let (app, lk, state) = setup_state(db.clone());
    let (owner, uid) = login_as(&app, "streamer1").await;
    let id = create(&app, &owner).await;
    go_live(&app, Some(&owner), id, json!({ "tabId": TAB_A })).await;
    let a = format!("u:{uid}:{TAB_A}");
    webhook(&app, participant("participant_joined", id, &a, true)).await;
    webhook(&app, participant("participant_left", id, &a, true)).await;
    assert_eq!(detail(&app, id).await["reconnecting"], true);

    // 같은 탭이 같은 신원의 시청(구독 전용) 토큰으로 들어와도 송출자 복귀가 아니다
    webhook(&app, participant("participant_joined", id, &a, false)).await;
    assert_eq!(detail(&app, id).await["reconnecting"], true);
    webhook(&app, participant("participant_left", id, &a, false)).await;
    assert_eq!(detail(&app, id).await["reconnecting"], true);

    // 유예 만료 때 LiveKit에 구독 전용으로만 있으면 송출자가 없는 것으로 보고 종료
    lk.participants
        .lock()
        .unwrap()
        .insert(format!("broadcast-{id}"), vec![member(&a, false)]);
    backdate_disconnect(&db, id, 61).await;
    check_limits(&state).await.unwrap();
    assert_eq!(detail(&app, id).await["status"], "ENDED");
    assert!(lk.removed_participants.lock().unwrap().is_empty());
}

/// 시청 토큰은 IP당 1분 30회(T45 감사). 제한은 방송 확인보다 먼저라 없는 방송 요청도 센다.
#[sqlx::test]
async fn live_viewer_token_rate_limited(db: PgPool) {
    let (app, _lk) = setup(db);
    let body = json!({ "tabId": TAB_A, "guestId": "guest-12345678" });
    for _ in 0..30 {
        let res = viewer_token(&app, None, 999_999, body.clone()).await;
        assert_eq!(res.status, StatusCode::NOT_FOUND, "{}", res.json);
    }
    let res = viewer_token(&app, None, 999_999, body).await;
    assert_eq!(res.status, StatusCode::TOO_MANY_REQUESTS, "{}", res.json);
    assert_eq!(code(&res), "RATE_LIMITED");
}
