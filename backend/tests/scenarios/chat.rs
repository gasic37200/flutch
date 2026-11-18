//! 채팅 시나리오 (features/04, api.md §4).

use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::common::{
    FakeLiveKit, ORIGIN, TestResponse, empty_req, json_req, login_as, send, test_state,
};
use crate::realtime::{connect, next_event};

const B: &str = "/api/v1/broadcasts";
const C: &str = "/api/v1/chats";

async fn new_broadcast(app: &Router, cookie: &str) -> i64 {
    let body = json!({ "title": "채팅 테스트", "category": "game" });
    let res = send(app, json_req("POST", B, Some(cookie), body)).await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.json);
    res.json["id"].as_i64().unwrap()
}

async fn go_live(app: &Router, cookie: &str, id: i64) {
    let body = json!({ "tabId": "tab-aaaaaaaa" });
    let res = send(
        app,
        json_req("POST", &format!("{B}/{id}/live"), Some(cookie), body),
    )
    .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
}

async fn end_live(app: &Router, cookie: &str, id: i64) {
    let res = send(
        app,
        empty_req("DELETE", &format!("{B}/{id}/live"), Some(cookie)),
    )
    .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
}

/// 방송 하나를 만들어 LIVE로 올린다.
async fn live_broadcast(app: &Router, cookie: &str) -> i64 {
    let id = new_broadcast(app, cookie).await;
    go_live(app, cookie, id).await;
    id
}

async fn post(app: &Router, cookie: Option<&str>, id: i64, body: &str) -> TestResponse {
    send(
        app,
        json_req(
            "POST",
            &format!("{B}/{id}/chats"),
            cookie,
            json!({ "body": body }),
        ),
    )
    .await
}

async fn patch(app: &Router, cookie: Option<&str>, id: i64, body: &str) -> TestResponse {
    send(
        app,
        json_req(
            "PATCH",
            &format!("{C}/{id}"),
            cookie,
            json!({ "body": body }),
        ),
    )
    .await
}

async fn delete(app: &Router, cookie: Option<&str>, id: i64) -> TestResponse {
    send(app, empty_req("DELETE", &format!("{C}/{id}"), cookie)).await
}

async fn list(app: &Router, id: i64, query: &str) -> TestResponse {
    send(
        app,
        empty_req("GET", &format!("{B}/{id}/chats{query}"), None),
    )
    .await
}

fn code(res: &TestResponse) -> &str {
    res.json["error"]["code"].as_str().unwrap()
}

fn chat_id(res: &TestResponse) -> i64 {
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.json);
    res.json["id"].as_i64().unwrap()
}

/// 레이트 리밋을 거치지 않고 채팅 행을 직접 넣는다.
async fn seed_chat(db: &PgPool, broadcast: i64, streamer: i64, user: i64, body: &str) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO chats (broadcast_id, streamer_id, user_id, body) VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(broadcast)
    .bind(streamer)
    .bind(user)
    .bind(body)
    .fetch_one(db)
    .await
    .unwrap()
}

#[sqlx::test]
async fn chat_live_only(db: PgPool) {
    let app = crate::common::test_app(db);
    let (owner, _) = login_as(&app, "streamer1").await;
    let (viewer, _) = login_as(&app, "viewer").await;

    let id = new_broadcast(&app, &owner).await;
    let res = post(&app, Some(&viewer), id, "대기 중").await;
    assert_eq!(
        (res.status, code(&res)),
        (StatusCode::CONFLICT, "BROADCAST_NOT_LIVE")
    );

    go_live(&app, &owner, id).await;
    let res = post(&app, Some(&viewer), id, "안녕하세요").await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.json);
    assert_eq!(res.json["body"], "안녕하세요");
    assert_eq!(res.json["author"]["nickname"], "시청자");
    assert_eq!(res.json["deleted"], false);

    end_live(&app, &owner, id).await;
    let res = post(&app, Some(&viewer), id, "종료 후").await;
    assert_eq!(
        (res.status, code(&res)),
        (StatusCode::CONFLICT, "BROADCAST_NOT_LIVE")
    );
    // 종료 후에도 기록은 읽힌다
    assert_eq!(
        list(&app, id, "").await.json["items"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[sqlx::test]
async fn chat_guest_read_only(db: PgPool) {
    let app = crate::common::test_app(db.clone());
    let (owner, owner_id) = login_as(&app, "streamer1").await;
    let (_, viewer_id) = login_as(&app, "viewer").await;
    let id = live_broadcast(&app, &owner).await;

    let res = post(&app, None, id, "비로그인").await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        patch(&app, None, 1, "x").await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(delete(&app, None, 1).await.status, StatusCode::UNAUTHORIZED);

    // 비로그인 조회: 오래된→최신, 커서 페이지네이션
    let ids = [
        seed_chat(&db, id, owner_id, viewer_id, "하나").await,
        seed_chat(&db, id, owner_id, viewer_id, "둘").await,
        seed_chat(&db, id, owner_id, viewer_id, "셋").await,
    ];
    let res = list(&app, id, "?limit=2").await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.json["hasMore"], true);
    let items = res.json["items"].as_array().unwrap();
    assert_eq!(
        [
            items[0]["id"].as_i64().unwrap(),
            items[1]["id"].as_i64().unwrap()
        ],
        [ids[1], ids[2]]
    );

    let res = list(&app, id, &format!("?limit=2&before={}", ids[1])).await;
    assert_eq!(res.json["hasMore"], false);
    assert_eq!(res.json["items"][0]["id"], ids[0]);

    assert_eq!(
        list(&app, id, "?limit=abc").await.status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(list(&app, 999_999, "").await.status, StatusCode::NOT_FOUND);

    // 숨긴 방송(종료 후 삭제)은 404
    end_live(&app, &owner, id).await;
    let res = send(
        &app,
        empty_req("DELETE", &format!("{B}/{id}"), Some(&owner)),
    )
    .await;
    assert_eq!(res.status, StatusCode::NO_CONTENT);
    assert_eq!(list(&app, id, "").await.status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn chat_edit_author_only_within_5min(db: PgPool) {
    let app = crate::common::test_app(db.clone());
    let (owner, _) = login_as(&app, "streamer1").await;
    let (viewer, _) = login_as(&app, "viewer").await;
    let (other, _) = login_as(&app, "streamer2").await;
    let id = live_broadcast(&app, &owner).await;
    let cid = chat_id(&post(&app, Some(&viewer), id, "오타").await);

    // 방송 소유자도 남의 글은 수정 못 한다
    for cookie in [&owner, &other] {
        let res = patch(&app, Some(cookie), cid, "바꿈").await;
        assert_eq!(
            (res.status, code(&res)),
            (StatusCode::FORBIDDEN, "CHAT_NOT_AUTHOR")
        );
    }
    let res = patch(&app, Some(&viewer), cid, "수정").await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    assert_eq!(
        (res.json["body"].as_str(), res.json["edited"].as_bool()),
        (Some("수정"), Some(true))
    );
    assert_eq!(
        patch(&app, Some(&viewer), cid, "").await.status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        patch(&app, Some(&viewer), 999_999, "x").await.status,
        StatusCode::NOT_FOUND
    );

    // 5분이 지나면 못 고친다
    sqlx::query("UPDATE chats SET created_at = now() - interval '6 minutes' WHERE id = $1")
        .bind(cid)
        .execute(&db)
        .await
        .unwrap();
    let res = patch(&app, Some(&viewer), cid, "늦음").await;
    assert_eq!(
        (res.status, code(&res)),
        (StatusCode::CONFLICT, "CHAT_EDIT_EXPIRED")
    );

    // 삭제된 글, 종료된 방송
    let cid2 = seed_chat_via_api(&app, &viewer, id, "두번째").await;
    assert_eq!(
        delete(&app, Some(&viewer), cid2).await.status,
        StatusCode::NO_CONTENT
    );
    let res = patch(&app, Some(&viewer), cid2, "x").await;
    assert_eq!(
        (res.status, code(&res)),
        (StatusCode::CONFLICT, "CHAT_DELETED")
    );

    let cid3 = seed_chat_via_api(&app, &viewer, id, "세번째").await;
    end_live(&app, &owner, id).await;
    let res = patch(&app, Some(&viewer), cid3, "x").await;
    assert_eq!(
        (res.status, code(&res)),
        (StatusCode::CONFLICT, "BROADCAST_NOT_LIVE")
    );
}

/// 1초 제한을 지키며 작성한다.
async fn seed_chat_via_api(app: &Router, cookie: &str, id: i64, body: &str) -> i64 {
    tokio::time::sleep(Duration::from_millis(1100)).await;
    chat_id(&post(app, Some(cookie), id, body).await)
}

#[sqlx::test]
async fn chat_delete_author_or_owner(db: PgPool) {
    let app = crate::common::test_app(db.clone());
    let (owner, _) = login_as(&app, "streamer1").await;
    let (viewer, _) = login_as(&app, "viewer").await;
    let (other, other_id) = login_as(&app, "streamer2").await;
    let id = live_broadcast(&app, &owner).await;
    let mine = chat_id(&post(&app, Some(&viewer), id, "내 글").await);
    let theirs = chat_id(&post(&app, Some(&other), id, "남의 글").await);

    // 다른 시청자는 못 지운다
    let res = delete(&app, Some(&other), mine).await;
    assert_eq!(
        (res.status, code(&res)),
        (StatusCode::FORBIDDEN, "CHAT_DELETE_FORBIDDEN")
    );
    assert_eq!(
        delete(&app, Some(&other), 999_999).await.status,
        StatusCode::NOT_FOUND
    );

    // 작성자는 지운다. 이미 삭제됐으면 다시 해도 204
    assert_eq!(
        delete(&app, Some(&viewer), mine).await.status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        delete(&app, Some(&viewer), mine).await.status,
        StatusCode::NO_CONTENT
    );

    // 방송 소유자는 남의 글을 지운다 — 방송이 끝난 뒤에도
    end_live(&app, &owner, id).await;
    assert_eq!(
        delete(&app, Some(&owner), theirs).await.status,
        StatusCode::NO_CONTENT
    );

    // 삭제된 글은 본문 없이 deleted만 내려간다(원문은 DB에 남는다)
    let res = list(&app, id, "").await;
    for item in res.json["items"].as_array().unwrap() {
        assert_eq!(
            (item["deleted"].as_bool(), item["body"].is_null()),
            (Some(true), true)
        );
    }
    let n: i64 =
        sqlx::query_scalar("SELECT count(*) FROM chats WHERE user_id = $1 AND body = '남의 글'")
            .bind(other_id)
            .fetch_one(&db)
            .await
            .unwrap();
    assert_eq!(n, 1);
}

#[sqlx::test]
async fn chat_rate_limit_and_repeat(db: PgPool) {
    let app = crate::common::test_app(db);
    let (owner, _) = login_as(&app, "streamer1").await;
    let (viewer, _) = login_as(&app, "viewer").await;
    let (other, _) = login_as(&app, "streamer2").await;
    let id = live_broadcast(&app, &owner).await;

    // 1초 1개: 연속 전송은 429, 다른 사용자는 영향 없음
    chat_id(&post(&app, Some(&viewer), id, "첫째").await);
    let res = post(&app, Some(&viewer), id, "바로 또").await;
    assert_eq!(
        (res.status, code(&res)),
        (StatusCode::TOO_MANY_REQUESTS, "RATE_LIMITED")
    );
    assert!(res.json["error"]["retryAfterSec"].as_i64().unwrap() >= 1);
    chat_id(&post(&app, Some(&other), id, "다른 사람").await);

    // 같은 메시지 30초 안 3번째는 거절(2번까지는 허용). 다른 사용자의 같은 말은 세지 않는다
    tokio::time::sleep(Duration::from_millis(1100)).await;
    chat_id(&post(&app, Some(&viewer), id, "도배").await);
    tokio::time::sleep(Duration::from_millis(1100)).await;
    chat_id(&post(&app, Some(&viewer), id, "도배").await);
    tokio::time::sleep(Duration::from_millis(1100)).await;
    let res = post(&app, Some(&viewer), id, "도배").await;
    assert_eq!(
        (res.status, code(&res)),
        (StatusCode::TOO_MANY_REQUESTS, "CHAT_REPEATED")
    );
    chat_id(&post(&app, Some(&other), id, "도배").await);

    // 길이: 200자까지, 빈 글·201자는 422
    tokio::time::sleep(Duration::from_millis(1100)).await;
    assert_eq!(
        post(&app, Some(&viewer), id, "   ").await.status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        post(&app, Some(&viewer), id, &"가".repeat(201))
            .await
            .status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    chat_id(&post(&app, Some(&viewer), id, &"가".repeat(200)).await);
}

#[sqlx::test]
async fn chat_profanity_masked_banned_word_blocked(db: PgPool) {
    let app = crate::common::test_app(db.clone());
    let (owner, _) = login_as(&app, "streamer1").await;
    let (viewer, viewer_id) = login_as(&app, "viewer").await;
    let id = live_broadcast(&app, &owner).await;

    // 욕설: 원문 저장, 응답·조회는 가림
    let res = post(&app, Some(&viewer), id, "이 개새끼야").await;
    let cid = chat_id(&res);
    assert_eq!(res.json["body"], "이 ***야");
    let raw: String = sqlx::query_scalar("SELECT body FROM chats WHERE id = $1")
        .bind(cid)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(raw, "이 개새끼야");
    assert_eq!(
        list(&app, id, "").await.json["items"][0]["body"],
        "이 ***야"
    );

    // 수정에도 같은 규칙
    let res = patch(&app, Some(&viewer), cid, "FUCK you").await;
    assert_eq!(res.json["body"], "*** you");

    // 금지어: 422, 저장 안 함(작성·수정 모두)
    tokio::time::sleep(Duration::from_millis(1100)).await;
    let res = post(&app, Some(&viewer), id, "바카라 하실 분").await;
    assert_eq!(
        (res.status, code(&res)),
        (StatusCode::UNPROCESSABLE_ENTITY, "CHAT_BANNED_WORD")
    );
    let res = patch(&app, Some(&viewer), cid, "토토사이트").await;
    assert_eq!(
        (res.status, code(&res)),
        (StatusCode::UNPROCESSABLE_ENTITY, "CHAT_BANNED_WORD")
    );
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM chats WHERE user_id = $1")
        .bind(viewer_id)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(n, 1);
}

#[sqlx::test]
async fn chat_banned_user_cannot_write(db: PgPool) {
    let app = crate::common::test_app(db.clone());
    let (owner, owner_id) = login_as(&app, "streamer1").await;
    let (viewer, viewer_id) = login_as(&app, "viewer").await;
    let (_, other_id) = login_as(&app, "streamer2").await;
    let id = live_broadcast(&app, &owner).await;
    let cid = chat_id(&post(&app, Some(&viewer), id, "차단 전").await);

    let ban = |streamer: i64, secs: Option<f64>| {
        let db = db.clone();
        async move {
            sqlx::query(
                "INSERT INTO bans (streamer_id, user_id, until)
                 VALUES ($1, $2, now() + make_interval(secs => $3::float8))
                 ON CONFLICT (streamer_id, user_id) DO UPDATE SET until = EXCLUDED.until",
            )
            .bind(streamer)
            .bind(viewer_id)
            .bind(secs)
            .execute(&db)
            .await
            .unwrap();
        }
    };

    // 다른 스트리머의 차단은 이 방송에 영향 없음
    ban(other_id, None).await;
    tokio::time::sleep(Duration::from_millis(1100)).await;
    chat_id(&post(&app, Some(&viewer), id, "영향 없음").await);

    // 임시차단: 작성·수정 403 + retryAfterSec, 삭제(본인 글)는 가능
    ban(owner_id, Some(600.0)).await;
    tokio::time::sleep(Duration::from_millis(1100)).await;
    let res = post(&app, Some(&viewer), id, "임시차단").await;
    assert_eq!(
        (res.status, code(&res)),
        (StatusCode::FORBIDDEN, "MODERATION_TIMED_OUT")
    );
    let retry = res.json["error"]["retryAfterSec"].as_i64().unwrap();
    assert!((1..=600).contains(&retry));
    let res = patch(&app, Some(&viewer), cid, "수정").await;
    assert_eq!(
        (res.status, code(&res)),
        (StatusCode::FORBIDDEN, "MODERATION_TIMED_OUT")
    );

    // 영구 밴
    ban(owner_id, None).await;
    let res = post(&app, Some(&viewer), id, "영구").await;
    assert_eq!(
        (res.status, code(&res)),
        (StatusCode::FORBIDDEN, "MODERATION_BANNED")
    );
    let res = patch(&app, Some(&viewer), cid, "수정").await;
    assert_eq!(
        (res.status, code(&res)),
        (StatusCode::FORBIDDEN, "MODERATION_BANNED")
    );
    assert_eq!(
        delete(&app, Some(&viewer), cid).await.status,
        StatusCode::NO_CONTENT
    );

    // 지난 임시차단은 풀린 것
    ban(owner_id, Some(-60.0)).await;
    chat_id(&post(&app, Some(&viewer), id, "풀림").await);
}

#[sqlx::test]
async fn chat_badges_streamer_and_donor(db: PgPool) {
    let app = crate::common::test_app(db.clone());
    let (owner, _) = login_as(&app, "streamer1").await;
    let (viewer, viewer_id) = login_as(&app, "viewer").await;
    let (other, _) = login_as(&app, "streamer2").await;
    let id = live_broadcast(&app, &owner).await;

    assert_eq!(
        chat_id_badges(&post(&app, Some(&owner), id, "방송 시작").await),
        json!(["streamer"])
    );
    assert_eq!(
        chat_id_badges(&post(&app, Some(&viewer), id, "후원 전").await),
        json!([])
    );

    // 익명 후원만 → 배지 없음(신원 노출 방지, 사용자 승인 11/18). 공개 후원 → donor 배지. 원장·후원 행은 직접 넣는다
    let donate = |anonymous: bool| {
        let db = db.clone();
        async move {
            let ledger: i64 = sqlx::query_scalar(
                "INSERT INTO ledger_entries (user_id, type, amount, balance_after, broadcast_id)
                 VALUES ($1, 'donation', -1000, 0, $2) RETURNING id",
            )
            .bind(viewer_id)
            .bind(id)
            .fetch_one(&db)
            .await
            .unwrap();
            sqlx::query(
                "INSERT INTO donations (broadcast_id, donor_id, kind, amount, anonymous, ledger_entry_id)
                 VALUES ($1, $2, 'donation', 1000, $3, $4)",
            )
            .bind(id)
            .bind(viewer_id)
            .bind(anonymous)
            .bind(ledger)
            .execute(&db)
            .await
            .unwrap();
        }
    };
    donate(true).await;
    let items = list(&app, id, "").await.json["items"].clone();
    assert_eq!(items[1]["badges"], json!([]));
    tokio::time::sleep(Duration::from_millis(1100)).await;
    assert_eq!(
        chat_id_badges(&post(&app, Some(&viewer), id, "익명 후원 후").await),
        json!([])
    );
    donate(false).await;
    let items = list(&app, id, "").await.json["items"].clone();
    assert_eq!(items[1]["badges"], json!(["donor"]));
    tokio::time::sleep(Duration::from_millis(1100)).await;
    assert_eq!(
        chat_id_badges(&post(&app, Some(&viewer), id, "후원 후").await),
        json!(["donor"])
    );
    assert_eq!(
        chat_id_badges(&post(&app, Some(&other), id, "남").await),
        json!([])
    );
}

fn chat_id_badges(res: &TestResponse) -> Value {
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.json);
    res.json["badges"].clone()
}

#[sqlx::test]
async fn chat_events_published(db: PgPool) {
    let state = test_state(db, Arc::new(FakeLiveKit::default()));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = flutch::app(state.clone());
    tokio::spawn(async move { axum::serve(listener, server).await.unwrap() });
    let app = flutch::app(state);

    let (owner, _) = login_as(&app, "streamer1").await;
    let (viewer, _) = login_as(&app, "viewer").await;
    let id = live_broadcast(&app, &owner).await;
    let mut ws = connect(addr, id, Some(ORIGIN), None).await.unwrap();

    let cid = chat_id(&post(&app, Some(&viewer), id, "이벤트 개새끼").await);
    let ev = next_event(&mut ws).await;
    assert_eq!(
        (ev["type"].as_str(), ev["data"]["id"].as_i64()),
        (Some("chat.created"), Some(cid))
    );
    assert_eq!(ev["data"]["body"], "이벤트 ***");

    patch(&app, Some(&viewer), cid, "고침").await;
    let ev = next_event(&mut ws).await;
    assert_eq!(
        (ev["type"].as_str(), ev["data"]["edited"].as_bool()),
        (Some("chat.updated"), Some(true))
    );

    delete(&app, Some(&owner), cid).await;
    let ev = next_event(&mut ws).await;
    assert_eq!(ev, json!({ "type": "chat.deleted", "data": { "id": cid } }));
}
