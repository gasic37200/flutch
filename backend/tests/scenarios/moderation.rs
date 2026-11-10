//! 모더레이션 시나리오 (features/05, api.md §5).

use std::sync::Arc;

use axum::Router;
use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::common::{
    FakeLiveKit, ORIGIN, TestResponse, empty_req, json_req, login_as, send, test_state,
};
use crate::realtime::{connect, next_event};

const B: &str = "/api/v1/broadcasts";

async fn live_broadcast(app: &Router, cookie: &str) -> i64 {
    let body = json!({ "title": "차단 테스트", "category": "game" });
    let res = send(app, json_req("POST", B, Some(cookie), body)).await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.json);
    let id = res.json["id"].as_i64().unwrap();
    let body = json!({ "tabId": "tab-aaaaaaaa" });
    let res = send(
        app,
        json_req("POST", &format!("{B}/{id}/live"), Some(cookie), body),
    )
    .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    id
}

async fn end_live(app: &Router, cookie: &str, id: i64) {
    let res = send(
        app,
        empty_req("DELETE", &format!("{B}/{id}/live"), Some(cookie)),
    )
    .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
}

async fn post_chat(app: &Router, cookie: &str, id: i64, body: &str) -> TestResponse {
    let b = json!({ "body": body });
    send(
        app,
        json_req("POST", &format!("{B}/{id}/chats"), Some(cookie), b),
    )
    .await
}

async fn ban(
    app: &Router,
    cookie: Option<&str>,
    streamer: i64,
    target: i64,
    duration: &str,
) -> TestResponse {
    let path = format!("/api/v1/users/{streamer}/bans/{target}");
    let b = json!({ "duration": duration });
    send(app, json_req("PUT", &path, cookie, b)).await
}

async fn get(app: &Router, cookie: Option<&str>, path: &str) -> TestResponse {
    send(app, empty_req("GET", path, cookie)).await
}

fn code(res: &TestResponse) -> &str {
    res.json["error"]["code"].as_str().unwrap()
}

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

async fn detail_ban(app: &Router, cookie: &str, id: i64) -> Value {
    let res = get(app, Some(cookie), &format!("{B}/{id}")).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    res.json["viewer"]["ban"].clone()
}

#[sqlx::test]
async fn ban_blocks_chat_across_broadcasts(db: PgPool) {
    let app = crate::common::test_app(db);
    let (owner, oid) = login_as(&app, "streamer1").await;
    let (viewer, vid) = login_as(&app, "viewer").await;
    let b1 = live_broadcast(&app, &owner).await;
    assert_eq!(
        post_chat(&app, &viewer, b1, "차단 전").await.status,
        StatusCode::CREATED
    );

    let res = ban(&app, Some(&owner), oid, vid, "permanent").await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    assert_eq!(res.json["permanent"], true);
    assert_eq!(res.json["until"], Value::Null);

    end_live(&app, &owner, b1).await;
    let b2 = live_broadcast(&app, &owner).await;
    let res = post_chat(&app, &viewer, b2, "다른 방송").await;
    assert_eq!(
        (res.status, code(&res)),
        (StatusCode::FORBIDDEN, "MODERATION_BANNED")
    );
}

#[sqlx::test]
async fn timeout_expires_automatically(db: PgPool) {
    let app = crate::common::test_app(db.clone());
    let (owner, oid) = login_as(&app, "streamer1").await;
    let (viewer, vid) = login_as(&app, "viewer").await;
    let id = live_broadcast(&app, &owner).await;

    let res = ban(&app, Some(&owner), oid, vid, "1m").await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    assert_eq!(res.json["permanent"], false);
    assert_ne!(res.json["until"], Value::Null);
    assert_eq!(
        post_chat(&app, &viewer, id, "x").await.status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(detail_ban(&app, &viewer, id).await["permanent"], false);
    let list = get(&app, Some(&owner), &format!("/api/v1/users/{oid}/bans")).await;
    assert_eq!(list.json["items"].as_array().unwrap().len(), 1);

    // 시간 조작: 만료 시각을 과거로
    sqlx::query("UPDATE bans SET until = now() - interval '1 second'")
        .execute(&db)
        .await
        .unwrap();
    assert_eq!(
        post_chat(&app, &viewer, id, "풀렸어요").await.status,
        StatusCode::CREATED
    );
    assert_eq!(detail_ban(&app, &viewer, id).await, Value::Null);
    let list = get(&app, Some(&owner), &format!("/api/v1/users/{oid}/bans")).await;
    assert!(list.json["items"].as_array().unwrap().is_empty());
}

#[sqlx::test]
async fn ban_effects_matrix(db: PgPool) {
    let app = crate::common::test_app(db.clone());
    let (owner, oid) = login_as(&app, "streamer1").await;
    let (viewer, vid) = login_as(&app, "viewer").await;
    let id = live_broadcast(&app, &owner).await;
    let own = post_chat(&app, &viewer, id, "내 글").await.json["id"]
        .as_i64()
        .unwrap();
    let token = |c: &str| {
        send(
            &app,
            json_req(
                "POST",
                &format!("{B}/{id}/viewer-token"),
                Some(c),
                json!({ "tabId": "tab-bbbbbbbb" }),
            ),
        )
    };
    let edit = |c: &str| {
        send(
            &app,
            json_req(
                "PATCH",
                &format!("/api/v1/chats/{own}"),
                Some(c),
                json!({ "body": "수정" }),
            ),
        )
    };

    // 임시차단: 작성·수정 불가(남은 시간 포함), 시청·조회 가능
    ban(&app, Some(&owner), oid, vid, "10m").await;
    let res = post_chat(&app, &viewer, id, "x").await;
    assert_eq!(
        (res.status, code(&res)),
        (StatusCode::FORBIDDEN, "MODERATION_TIMED_OUT")
    );
    assert!(res.json["error"]["retryAfterSec"].as_i64().unwrap() > 0);
    assert_eq!(code(&edit(&viewer).await), "MODERATION_TIMED_OUT");
    assert_eq!(token(&viewer).await.status, StatusCode::OK);
    assert_eq!(
        get(&app, Some(&viewer), &format!("{B}/{id}/chats"))
            .await
            .status,
        StatusCode::OK
    );

    // 영구 밴: 작성·수정 불가, 시청·조회 가능, 자기 글 삭제는 가능
    // (API로 밴하면 현재 글이 삭제돼 수정이 CHAT_DELETED가 되므로, 수정 검사를 위해 행만 영구로 바꾼다)
    sqlx::query("UPDATE bans SET until = NULL")
        .execute(&db)
        .await
        .unwrap();
    let res = post_chat(&app, &viewer, id, "x").await;
    assert_eq!(
        (res.status, code(&res)),
        (StatusCode::FORBIDDEN, "MODERATION_BANNED")
    );
    assert_eq!(code(&edit(&viewer).await), "MODERATION_BANNED");
    assert_eq!(token(&viewer).await.status, StatusCode::OK);
    assert_eq!(detail_ban(&app, &viewer, id).await["permanent"], true);
    let res = send(
        &app,
        empty_req("DELETE", &format!("/api/v1/chats/{own}"), Some(&viewer)),
    )
    .await;
    assert_eq!(res.status, StatusCode::NO_CONTENT);
    // shortcut: 후원·유료 클러치 행은 T22·T19가 자기 시나리오에서 같은 `chat::ban_state`로 확인한다.
}

#[sqlx::test]
async fn ban_only_by_owner(db: PgPool) {
    let app = crate::common::test_app(db);
    let (owner, oid) = login_as(&app, "streamer1").await;
    let (other, _) = login_as(&app, "streamer2").await;
    let (viewer, vid) = login_as(&app, "viewer").await;

    let res = ban(&app, None, oid, vid, "1m").await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    for cookie in [&other, &viewer] {
        let res = ban(&app, Some(cookie), oid, vid, "1m").await;
        assert_eq!(
            (res.status, code(&res)),
            (StatusCode::FORBIDDEN, "MODERATION_NOT_OWNER")
        );
        let res = get(&app, Some(cookie), &format!("/api/v1/users/{oid}/bans")).await;
        assert_eq!(res.status, StatusCode::FORBIDDEN);
        let res = send(
            &app,
            empty_req(
                "DELETE",
                &format!("/api/v1/users/{oid}/bans/{vid}"),
                Some(cookie),
            ),
        )
        .await;
        assert_eq!(res.status, StatusCode::FORBIDDEN);
        let res = get(
            &app,
            Some(cookie),
            &format!("/api/v1/users/{oid}/viewers/{vid}/chats"),
        )
        .await;
        assert_eq!(res.status, StatusCode::FORBIDDEN);
    }
    let res = get(&app, None, &format!("/api/v1/users/{oid}/bans")).await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        ban(&app, Some(&owner), oid, vid, "1m").await.status,
        StatusCode::OK
    );
}

#[sqlx::test]
async fn permanent_ban_hides_current_messages(db: PgPool) {
    let state = test_state(db.clone(), Arc::new(FakeLiveKit::default()));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = flutch::app(state.clone());
    tokio::spawn(async move { axum::serve(listener, server).await.unwrap() });
    let app = flutch::app(state);

    let (owner, oid) = login_as(&app, "streamer1").await;
    let (viewer, vid) = login_as(&app, "viewer").await;
    let (_, other_id) = login_as(&app, "streamer2").await;
    let id = live_broadcast(&app, &owner).await;
    post_chat(&app, &viewer, id, "하나").await;
    post_chat(&app, &viewer, id, "둘").await;
    let kept = seed_chat(&db, id, oid, other_id, "남는 글").await;
    let mut ws = connect(addr, id, Some(ORIGIN), None).await.unwrap();

    // 임시차단은 메시지를 지우지 않고 알림만 보낸다
    ban(&app, Some(&owner), oid, vid, "10m").await;
    let ev = next_event(&mut ws).await;
    assert_eq!(ev["type"], "chat.system");
    assert_eq!(ev["data"]["kind"], "timeout");
    assert_eq!(ev["data"]["user"]["id"], vid);
    assert_ne!(ev["data"]["until"], Value::Null);
    let chats = get(&app, None, &format!("{B}/{id}/chats")).await;
    assert!(
        chats.json["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|c| c["deleted"] == false)
    );

    ban(&app, Some(&owner), oid, vid, "permanent").await;
    let ev = next_event(&mut ws).await;
    assert_eq!(ev["type"], "chat.system");
    assert_eq!(ev["data"]["kind"], "ban");
    assert_eq!(ev["data"]["until"], Value::Null);

    let chats = get(&app, None, &format!("{B}/{id}/chats")).await;
    for c in chats.json["items"].as_array().unwrap() {
        let mine = c["author"]["id"] == vid;
        assert_eq!(c["deleted"], mine, "{c}");
        if mine {
            assert_eq!(c["body"], Value::Null);
        } else {
            assert_eq!(c["id"], kept);
        }
    }
}

#[sqlx::test]
async fn ban_history_unban_and_validation(db: PgPool) {
    let app = crate::common::test_app(db.clone());
    let (owner, oid) = login_as(&app, "streamer1").await;
    let (_, vid) = login_as(&app, "viewer").await;
    let (_, sid) = login_as(&app, "streamer2").await;
    let path = format!("/api/v1/users/{oid}/bans");

    let res = ban(&app, Some(&owner), oid, oid, "1m").await;
    assert_eq!(
        (res.status, code(&res)),
        (StatusCode::UNPROCESSABLE_ENTITY, "MODERATION_SELF")
    );
    assert_eq!(
        ban(&app, Some(&owner), oid, 999_999, "1m").await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        ban(&app, Some(&owner), oid, vid, "2h").await.status,
        StatusCode::UNPROCESSABLE_ENTITY
    );

    // 덮어쓰기(timeout → ban)는 행 1개, 이력은 둘
    ban(&app, Some(&owner), oid, vid, "1h").await;
    ban(&app, Some(&owner), oid, vid, "permanent").await;
    ban(&app, Some(&owner), oid, sid, "1m").await;
    let res = get(&app, Some(&owner), &path).await;
    let items = res.json["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["user"]["id"], sid, "최신순");
    assert_eq!(items[1]["permanent"], true);

    let res = get(&app, Some(&owner), &format!("{path}?limit=1")).await;
    assert_eq!(res.json["items"].as_array().unwrap().len(), 1);
    let cursor = res.json["nextCursor"].as_str().unwrap().to_owned();
    let res = get(
        &app,
        Some(&owner),
        &format!("{path}?limit=1&cursor={cursor}"),
    )
    .await;
    assert_eq!(res.json["items"][0]["user"]["id"], vid);
    assert_eq!(res.json["nextCursor"], Value::Null);

    // 해제: 행 삭제 + unban 이력, 없어도 204(이력은 안 남김)
    let del = || {
        send(
            &app,
            empty_req(
                "DELETE",
                &format!("/api/v1/users/{oid}/bans/{vid}"),
                Some(&owner),
            ),
        )
    };
    assert_eq!(del().await.status, StatusCode::NO_CONTENT);
    assert_eq!(del().await.status, StatusCode::NO_CONTENT);
    let actions: Vec<String> =
        sqlx::query_scalar("SELECT action FROM ban_events WHERE user_id = $1 ORDER BY id")
            .bind(vid)
            .fetch_all(&db)
            .await
            .unwrap();
    assert_eq!(actions, ["timeout", "ban", "unban"]);

    // 이력은 추가만
    assert!(
        sqlx::query("DELETE FROM ban_events")
            .execute(&db)
            .await
            .is_err()
    );
}

#[sqlx::test]
async fn viewer_chat_history(db: PgPool) {
    let app = crate::common::test_app(db.clone());
    let (owner, oid) = login_as(&app, "streamer1").await;
    let (_, vid) = login_as(&app, "viewer").await;
    let (_, sid) = login_as(&app, "streamer2").await;
    let b1 = live_broadcast(&app, &owner).await;
    end_live(&app, &owner, b1).await;
    let b2 = live_broadcast(&app, &owner).await;
    let (other_owner, _) = login_as(&app, "streamer2").await;
    let b3 = live_broadcast(&app, &other_owner).await;

    let a = seed_chat(&db, b1, oid, vid, "첫 방송").await;
    let b = seed_chat(&db, b2, oid, vid, "둘째 방송").await;
    seed_chat(&db, b2, oid, sid, "다른 사람").await;
    seed_chat(&db, b3, sid, vid, "남의 방송").await;
    let path = format!("/api/v1/users/{oid}/viewers/{vid}/chats");

    let res = get(&app, Some(&owner), &path).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    let ids: Vec<_> = res.json["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["id"].as_i64().unwrap())
        .collect();
    assert_eq!(ids, [a, b], "내 방송의 이 시청자 채팅만, 오래된→최신");

    let res = get(&app, Some(&owner), &format!("{path}?limit=1")).await;
    assert_eq!(res.json["hasMore"], true);
    assert_eq!(res.json["items"][0]["id"], b);
    let res = get(&app, Some(&owner), &format!("{path}?limit=1&before={b}")).await;
    assert_eq!(res.json["items"][0]["id"], a);
    assert_eq!(res.json["hasMore"], false);

    let res = get(
        &app,
        Some(&owner),
        &format!("/api/v1/users/{oid}/viewers/999999/chats"),
    )
    .await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
}
