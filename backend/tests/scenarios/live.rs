//! 송출·시청 시나리오 (features/03, api.md §3-1).

use std::sync::Arc;

use axum::Router;
use axum::http::StatusCode;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use hmac::{Hmac, KeyInit, Mac};
use serde_json::{Value, json};
use sha2::Sha256;
use sqlx::PgPool;

use crate::common::{FakeLiveKit, TestResponse, empty_req, json_req, login_as, send, test_state};

const B: &str = "/api/v1/broadcasts";
const TAB_A: &str = "tab-aaaaaaaa";
const TAB_B: &str = "tab-bbbbbbbb";

fn setup(db: PgPool) -> (Router, Arc<FakeLiveKit>) {
    let lk = Arc::new(FakeLiveKit::default());
    (flutch::app(test_state(db, lk.clone())), lk)
}

async fn create(app: &Router, cookie: &str) -> i64 {
    let body = json!({ "title": "송출 테스트", "category": "game" });
    let res = send(app, json_req("POST", B, Some(cookie), body)).await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.json);
    res.json["id"].as_i64().unwrap()
}

async fn go_live(app: &Router, cookie: Option<&str>, id: i64, body: Value) -> TestResponse {
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

    // 밀려난 기기가 그냥 다시 시작하면 다시 확인
    let res = go_live(&app, Some(&owner), id, json!({ "tabId": TAB_A })).await;
    assert_eq!(code(&res), "LIVE_PUBLISHER_ACTIVE");

    // 송출자가 끊겨 유예 중이면(T08b 웹훅이 기록) 확인 없이 이어서 송출
    sqlx::query("UPDATE broadcasts SET disconnected_at = now() WHERE id = $1")
        .bind(id)
        .execute(&db)
        .await
        .unwrap();
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
