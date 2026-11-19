//! 방송 관리 시나리오 (features/02, api.md §3).

use std::sync::Arc;

use axum::Router;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use flutch::AppState;
use flutch::broadcast::{EndReason, end_live, start_live};
use flutch::error::AppError;
use serde_json::json;
use sqlx::PgPool;

use crate::common::{
    FakeLiveKit, TestResponse, empty_req, json_req, login_as, send, test_app, test_state,
};

const B: &str = "/api/v1/broadcasts";

async fn create(app: &Router, cookie: &str, title: &str) -> i64 {
    create_tagged(app, cookie, title, json!([])).await
}

async fn create_tagged(app: &Router, cookie: &str, title: &str, tags: serde_json::Value) -> i64 {
    let body = json!({ "title": title, "category": "game", "tags": tags });
    let res = send(app, json_req("POST", B, Some(cookie), body)).await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.json);
    res.json["id"].as_i64().unwrap()
}

async fn get(app: &Router, path: &str, cookie: Option<&str>) -> TestResponse {
    send(app, empty_req("GET", path, cookie)).await
}

async fn patch(app: &Router, cookie: &str, id: i64, body: serde_json::Value) -> TestResponse {
    send(
        app,
        json_req("PATCH", &format!("{B}/{id}"), Some(cookie), body),
    )
    .await
}

async fn delete(app: &Router, cookie: Option<&str>, id: i64) -> TestResponse {
    send(app, empty_req("DELETE", &format!("{B}/{id}"), cookie)).await
}

fn code(res: &TestResponse) -> &str {
    res.json["error"]["code"].as_str().unwrap()
}

/// 전이 함수가 돌려준 에러의 (상태, 코드).
async fn err_of(e: AppError) -> (StatusCode, String) {
    let res = e.into_response();
    let status = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    (status, json["error"]["code"].as_str().unwrap().to_owned())
}

fn ids(res: &TestResponse) -> Vec<i64> {
    res.json["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["id"].as_i64().unwrap())
        .collect()
}

fn setup(db: PgPool) -> (Router, Arc<AppState>, Arc<FakeLiveKit>) {
    let lk = Arc::new(FakeLiveKit::default());
    let state = test_state(db, lk.clone());
    (flutch::app(state.clone()), state, lk)
}

#[sqlx::test]
async fn broadcast_owner_crud(db: PgPool) {
    let app = test_app(db);
    let (cookie, uid) = login_as(&app, "streamer1").await;

    let body = json!({ "title": "  첫 방송  ", "category": "game", "tags": ["롤", "롤", "랭크"] });
    let res = send(&app, json_req("POST", B, Some(&cookie), body)).await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.json);
    assert_eq!(res.json["status"], "READY");
    assert_eq!(res.json["title"], "첫 방송");
    assert_eq!(res.json["tags"], json!(["롤", "랭크"]));
    assert_eq!(res.json["record"], true);
    assert_eq!(res.json["pipCorner"], "bottom_right");
    assert_eq!(res.json["owner"]["id"], uid);
    assert_eq!(res.json["viewer"]["isOwner"], true);
    assert_eq!(res.json["gauge"]["phase"], "charging");
    assert!(res.json["thumbnailUrl"].is_null());
    let id = res.json["id"].as_i64().unwrap();

    let body = json!({ "title": "바뀐 제목", "record": false, "pipCorner": "top_left" });
    let res = patch(&app, &cookie, id, body).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    assert_eq!(res.json["title"], "바뀐 제목");
    assert_eq!(res.json["record"], false);
    assert_eq!(res.json["pipCorner"], "top_left");

    let mine = get(&app, "/api/v1/me/broadcasts", Some(&cookie)).await;
    assert_eq!(ids(&mine), vec![id]);

    // 검증: 길이·금지어·태그 개수·태그 길이
    for (body, field) in [
        (json!({ "title": "가", "category": "game" }), "title"),
        (
            json!({ "title": "바카라 방송", "category": "game" }),
            "title",
        ),
        (
            json!({ "title": "정상 제목", "category": "game", "tags": ["a", "b", "c", "d"] }),
            "tags",
        ),
        (
            json!({ "title": "정상 제목", "category": "game", "tags": ["열한글자열한글자열한열"] }),
            "tags",
        ),
    ] {
        let res = send(&app, json_req("POST", B, Some(&cookie), body)).await;
        assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(
            res.json["error"]["fields"][field].is_string(),
            "{}",
            res.json
        );
    }
    let body = json!({ "title": "정상 제목", "category": "nope" });
    let res = send(&app, json_req("POST", B, Some(&cookie), body)).await;
    assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY);

    assert_eq!(
        delete(&app, Some(&cookie), id).await.status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        get(&app, &format!("{B}/{id}"), None).await.status,
        StatusCode::NOT_FOUND
    );
    assert!(ids(&get(&app, "/api/v1/me/broadcasts", Some(&cookie)).await).is_empty());
}

#[sqlx::test]
async fn broadcast_other_user_forbidden(db: PgPool) {
    let app = test_app(db);
    let (owner, _) = login_as(&app, "streamer1").await;
    let (other, _) = login_as(&app, "streamer2").await;
    let id = create(&app, &owner, "남의 방송").await;

    let res = patch(&app, &other, id, json!({ "title": "가로채기" })).await;
    assert_eq!(
        (res.status, code(&res)),
        (StatusCode::FORBIDDEN, "BROADCAST_NOT_OWNER")
    );
    let res = delete(&app, Some(&other), id).await;
    assert_eq!(
        (res.status, code(&res)),
        (StatusCode::FORBIDDEN, "BROADCAST_NOT_OWNER")
    );

    // 비로그인은 401, 없는 방송은 권한보다 404가 먼저
    let body = json!({ "title": "익명 방송", "category": "game" });
    let res = send(&app, json_req("POST", B, None, body)).await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    let req = json_req(
        "PATCH",
        &format!("{B}/{id}"),
        None,
        json!({ "title": "익명 수정" }),
    );
    assert_eq!(send(&app, req).await.status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        delete(&app, None, id).await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        get(&app, "/api/v1/me/broadcasts", None).await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        delete(&app, Some(&other), id + 999).await.status,
        StatusCode::NOT_FOUND
    );
}

#[sqlx::test]
async fn broadcast_public_read(db: PgPool) {
    let (app, state, _) = setup(db);
    let (c1, u1) = login_as(&app, "streamer1").await;
    let (c2, _) = login_as(&app, "streamer2").await;
    let ready = create(&app, &c1, "준비중 방송").await;
    let live1 = create_tagged(&app, &c1, "롤 랭크 방송", json!(["Valorant"])).await;
    let live2 = create(&app, &c2, "음악 방송").await;

    // READY는 목록에 없지만 링크(상세)로는 열린다. 비로그인이라 viewer는 null
    assert!(ids(&get(&app, B, None).await).is_empty());
    let res = get(&app, &format!("{B}/{ready}"), None).await;
    assert_eq!(res.status, StatusCode::OK);
    assert!(res.json["viewer"].is_null());

    start_live(&state.db, live1).await.unwrap();
    start_live(&state.db, live2).await.unwrap();
    let res = get(&app, B, None).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(ids(&res), vec![live2, live1]);
    assert_eq!(res.json["items"][0]["viewerCount"], 0);

    // 검색: 제목·태그(대소문자 무시)·닉네임, LIKE 문자는 글자로 취급 (한글은 퍼센트 인코딩)
    let q = |s: &str| format!("{B}?q={s}");
    assert_eq!(ids(&get(&app, &q("valorant"), None).await), vec![live1]);
    assert_eq!(
        ids(&get(&app, &q("%EC%9D%8C%EC%95%85"), None).await), // 음악
        vec![live2]
    );
    assert_eq!(
        ids(&get(&app, &q("%EC%8A%A4%ED%8A%B8%EB%A6%AC%EB%A8%B82"), None).await), // 스트리머2
        vec![live2]
    );
    assert!(ids(&get(&app, &q("%25"), None).await).is_empty());
    assert_eq!(
        ids(&get(&app, &format!("{B}?category=game&q=valorant"), None).await),
        vec![live1]
    );
    assert!(ids(&get(&app, &format!("{B}?category=fps"), None).await).is_empty());
    assert_eq!(
        get(&app, &format!("{B}?category=nope"), None).await.status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        get(&app, &format!("{B}?limit=abc"), None).await.status,
        StatusCode::UNPROCESSABLE_ENTITY
    );

    // 커서 페이지
    let p1 = get(&app, &format!("{B}?limit=1"), None).await;
    assert_eq!(ids(&p1), vec![live2]);
    let cursor = p1.json["nextCursor"].as_str().unwrap();
    let p2 = get(&app, &format!("{B}?limit=1&cursor={cursor}"), None).await;
    assert_eq!(ids(&p2), vec![live1]);
    assert!(p2.json["nextCursor"].is_null());

    // 스트리머 페이지: ENDED만
    end_live(&state, live1, EndReason::Owner).await.unwrap();
    let res = get(&app, &format!("/api/v1/users/{u1}/broadcasts"), None).await;
    assert_eq!(ids(&res), vec![live1]);
    assert_eq!(
        get(&app, "/api/v1/users/999999/broadcasts", None)
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(&app, &format!("{B}/999999"), None).await.status,
        StatusCode::NOT_FOUND
    );
}

#[sqlx::test]
async fn broadcast_state_transitions(db: PgPool) {
    let (app, state, lk) = setup(db);
    let (cookie, _) = login_as(&app, "streamer1").await;
    let a = create(&app, &cookie, "방송 A").await;
    let b = create(&app, &cookie, "방송 B").await;
    let invalid = (
        StatusCode::CONFLICT,
        "BROADCAST_INVALID_TRANSITION".to_owned(),
    );

    start_live(&state.db, a).await.unwrap();
    // 같은 사용자의 두 번째 LIVE, LIVE 재시작은 거절
    let e = start_live(&state.db, b).await.unwrap_err();
    assert_eq!(
        err_of(e).await,
        (StatusCode::CONFLICT, "BROADCAST_LIVE_LIMIT".to_owned())
    );
    let e = start_live(&state.db, a).await.unwrap_err();
    assert_eq!(err_of(e).await, invalid);
    // READY는 종료할 수 없다
    let e = end_live(&state, b, EndReason::Owner).await.unwrap_err();
    assert_eq!(err_of(e).await, invalid);
    assert!(lk.deleted_rooms.lock().unwrap().is_empty());

    end_live(&state, a, EndReason::Owner).await.unwrap();
    assert_eq!(
        *lk.deleted_rooms.lock().unwrap(),
        vec![format!("broadcast-{a}")]
    );
    // ENDED는 최종 상태: 재시작·재종료 409
    let e = start_live(&state.db, a).await.unwrap_err();
    assert_eq!(err_of(e).await, invalid);
    let e = end_live(&state, a, EndReason::Owner).await.unwrap_err();
    assert_eq!(err_of(e).await, invalid);
    // 없는 방송은 404
    let e = start_live(&state.db, a + 999).await.unwrap_err();
    assert_eq!(err_of(e).await.0, StatusCode::NOT_FOUND);

    // 앞 방송이 끝나면 다음 방송을 시작할 수 있다
    start_live(&state.db, b).await.unwrap();
    let res = get(&app, &format!("{B}/{b}"), None).await;
    assert_eq!(res.json["status"], "LIVE");
    assert!(res.json["startedAt"].is_string());
    assert_eq!(res.json["reconnecting"], false);
}

#[sqlx::test]
async fn broadcast_limits(db: PgPool) {
    let (app, state, _) = setup(db);
    let (c1, _) = login_as(&app, "streamer1").await;
    let (c2, _) = login_as(&app, "streamer2").await;

    let mut ids = Vec::new();
    for i in 0..5 {
        ids.push(create(&app, &c1, &format!("준비 방송 {i}")).await);
    }
    let body = json!({ "title": "여섯번째", "category": "game" });
    let res = send(&app, json_req("POST", B, Some(&c1), body.clone())).await;
    assert_eq!(
        (res.status, code(&res)),
        (StatusCode::CONFLICT, "BROADCAST_READY_LIMIT")
    );
    // 한도는 사용자별이고, LIVE로 넘어가거나 지우면 자리가 난다
    create(&app, &c2, "다른 사람 방송").await;
    start_live(&state.db, ids[0]).await.unwrap();
    let res = send(&app, json_req("POST", B, Some(&c1), body.clone())).await;
    assert_eq!(res.status, StatusCode::CREATED);
    let res = send(&app, json_req("POST", B, Some(&c1), body)).await;
    assert_eq!(res.status, StatusCode::CONFLICT);
    assert_eq!(
        delete(&app, Some(&c1), ids[1]).await.status,
        StatusCode::NO_CONTENT
    );

    // LIVE: 제목·카테고리·태그·PIP만, 녹화 여부는 잠김
    let live = ids[0];
    let body = json!({ "title": "방송 중 제목", "category": "fps", "tags": ["새태그"], "pipCorner": "top_right" });
    let res = patch(&app, &c1, live, body).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    assert_eq!(res.json["category"], "fps");
    let res = patch(&app, &c1, live, json!({ "record": false })).await;
    assert_eq!(
        (res.status, code(&res)),
        (StatusCode::CONFLICT, "BROADCAST_FIELD_LOCKED")
    );

    // ENDED: 제목만
    end_live(&state, live, EndReason::Owner).await.unwrap();
    let res = patch(&app, &c1, live, json!({ "title": "끝난 방송" })).await;
    assert_eq!(res.status, StatusCode::OK);
    for body in [
        json!({ "category": "game" }),
        json!({ "tags": [] }),
        json!({ "pipCorner": "top_left" }),
        json!({ "record": true }),
    ] {
        let res = patch(&app, &c1, live, body).await;
        assert_eq!(
            (res.status, code(&res)),
            (StatusCode::CONFLICT, "BROADCAST_FIELD_LOCKED")
        );
    }
    // 검증(422)이 잠금(409)보다 먼저
    let res = patch(&app, &c1, live, json!({ "title": "가", "record": true })).await;
    assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn broadcast_delete_live_ends_first(db: PgPool) {
    let (app, state, lk) = setup(db.clone());
    let (cookie, _) = login_as(&app, "streamer1").await;
    let id = create(&app, &cookie, "삭제될 방송").await;
    start_live(&state.db, id).await.unwrap();

    assert_eq!(
        delete(&app, Some(&cookie), id).await.status,
        StatusCode::NO_CONTENT
    );
    let (status, hidden): (String, bool) =
        sqlx::query_as("SELECT status, hidden_at IS NOT NULL FROM broadcasts WHERE id = $1")
            .bind(id)
            .fetch_one(&db)
            .await
            .unwrap();
    assert_eq!((status.as_str(), hidden), ("ENDED", true));
    assert_eq!(
        *lk.deleted_rooms.lock().unwrap(),
        vec![format!("broadcast-{id}")]
    );
    assert_eq!(
        get(&app, &format!("{B}/{id}"), None).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        delete(&app, Some(&cookie), id).await.status,
        StatusCode::NOT_FOUND
    );

    // ENDED 삭제: 숨김 + 녹화 deleted
    let id2 = create(&app, &cookie, "끝난 방송").await;
    start_live(&state.db, id2).await.unwrap();
    end_live(&state, id2, EndReason::Owner).await.unwrap();
    sqlx::query("INSERT INTO recordings (broadcast_id) VALUES ($1)")
        .bind(id2)
        .execute(&db)
        .await
        .unwrap();
    assert_eq!(
        delete(&app, Some(&cookie), id2).await.status,
        StatusCode::NO_CONTENT
    );
    let rec: String = sqlx::query_scalar("SELECT status FROM recordings WHERE broadcast_id = $1")
        .bind(id2)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(rec, "deleted");
    assert!(ids(&get(&app, "/api/v1/me/broadcasts", Some(&cookie)).await).is_empty());
}

#[sqlx::test]
async fn broadcast_persists_after_restart(db: PgPool) {
    let (app, state, _) = setup(db.clone());
    let (cookie, _) = login_as(&app, "streamer1").await;
    let id = create(&app, &cookie, "재시작 방송").await;
    start_live(&state.db, id).await.unwrap();
    drop((app, state));

    // 같은 DB로 앱을 새로 만든다(서버 재시작). 세션 쿠키도 DB에 있어 그대로 유효하다.
    let app = test_app(db);
    assert_eq!(ids(&get(&app, B, None).await), vec![id]);
    let res = get(&app, &format!("{B}/{id}"), Some(&cookie)).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.json["title"], "재시작 방송");
    assert_eq!(res.json["status"], "LIVE");
    assert_eq!(res.json["viewer"]["isOwner"], true);
}

fn put_thumb(
    path: &str,
    cookie: Option<&str>,
    ctype: &str,
    body: Vec<u8>,
) -> axum::http::Request<axum::body::Body> {
    let mut req = axum::http::Request::builder()
        .method("PUT")
        .uri(path)
        .header("origin", crate::common::ORIGIN)
        .header("content-type", ctype);
    if let Some(c) = cookie {
        req = req.header("cookie", c);
    }
    req.body(axum::body::Body::from(body)).unwrap()
}

#[sqlx::test]
async fn broadcast_thumbnail_owner_live_only(db: PgPool) {
    let (app, state, _) = setup(db);
    let (owner, _) = login_as(&app, "streamer1").await;
    let (other, _) = login_as(&app, "streamer2").await;
    let id = create(&app, &owner, "썸네일 방송").await;
    let path = format!("{B}/{id}/thumbnail");
    let jpeg = || [vec![0xFF, 0xD8, 0xFF, 0xE0], vec![0; 100]].concat();

    let res = send(&app, put_thumb(&path, None, "image/jpeg", jpeg())).await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    let res = send(&app, put_thumb(&path, Some(&other), "image/jpeg", jpeg())).await;
    assert_eq!(res.status, StatusCode::FORBIDDEN);
    let res = send(&app, put_thumb(&path, Some(&owner), "image/jpeg", jpeg())).await;
    assert_eq!(
        (res.status, code(&res)),
        (StatusCode::CONFLICT, "BROADCAST_NOT_LIVE")
    );

    start_live(&state.db, id).await.unwrap();
    let big = [vec![0xFF, 0xD8, 0xFF], vec![0; 512 * 1024]].concat();
    for (ctype, body) in [
        ("image/jpeg", big),
        ("image/jpeg", b"not an image".to_vec()),
        ("image/png", jpeg()),
    ] {
        let res = send(&app, put_thumb(&path, Some(&owner), ctype, body)).await;
        assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY, "{}", res.json);
        assert_eq!(code(&res), "BROADCAST_THUMBNAIL_INVALID");
    }
    assert!(get(&app, &format!("{B}/{id}"), None).await.json["thumbnailUrl"].is_null());

    let res = send(&app, put_thumb(&path, Some(&owner), "image/jpeg", jpeg())).await;
    assert_eq!(res.status, StatusCode::NO_CONTENT);
    let url = get(&app, &format!("{B}/{id}"), None).await.json["thumbnailUrl"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(
        url.starts_with(&format!("/media/thumbs/{id}.jpg?v=")),
        "{url}"
    );
    let file = state.config.broadcast.thumb_dir.join(format!("{id}.jpg"));
    assert_eq!(std::fs::read(&file).unwrap(), jpeg());

    // 삭제(숨김)하면 파일도 정리한다
    assert_eq!(
        delete(&app, Some(&owner), id).await.status,
        StatusCode::NO_CONTENT
    );
    assert!(!file.exists());
}

#[sqlx::test]
async fn broadcast_overlay_layout_owner_only(db: PgPool) {
    let (app, state, _) = setup(db);
    let (owner, _) = login_as(&app, "streamer1").await;
    let (other, _) = login_as(&app, "streamer2").await;
    let id = create(&app, &owner, "배치 방송").await;
    assert!(get(&app, &format!("{B}/{id}"), None).await.json["overlayLayout"].is_null());

    let layout = json!({
        "alert": { "x": 50.0, "y": 10.0 },
        "cam": { "x": 0.0, "y": 0.0, "w": 22.0 },
        "chat": { "x": 2.0, "y": 94.0, "w": 30.0 }
    });
    // 다른 사용자는 403, 소유자는 READY에서 저장
    let res = patch(&app, &other, id, json!({ "overlayLayout": layout })).await;
    assert_eq!(
        (res.status, code(&res)),
        (StatusCode::FORBIDDEN, "BROADCAST_NOT_OWNER")
    );
    let res = patch(&app, &owner, id, json!({ "overlayLayout": layout })).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    assert_eq!(res.json["overlayLayout"], layout);

    // 범위 밖·폭 0은 422 fields, 필드가 빠지면 형식 오류 422
    for bad in [
        json!({ "alert": { "x": 101.0, "y": 0.0 }, "cam": layout["cam"], "chat": layout["chat"] }),
        json!({ "alert": layout["alert"], "cam": { "x": 0.0, "y": -1.0, "w": 22.0 }, "chat": layout["chat"] }),
        json!({ "alert": layout["alert"], "cam": layout["cam"], "chat": { "x": 0.0, "y": 0.0, "w": 0.0 } }),
    ] {
        let res = patch(&app, &owner, id, json!({ "overlayLayout": bad })).await;
        assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(res.json["error"]["fields"]["overlayLayout"].is_string(), "{}", res.json);
    }
    let bad = json!({ "alert": layout["alert"], "cam": layout["cam"] });
    let res = patch(&app, &owner, id, json!({ "overlayLayout": bad })).await;
    assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(get(&app, &format!("{B}/{id}"), None).await.json["overlayLayout"], layout);

    // 새 방송은 소유자의 마지막 배치를 이어받는다(다른 사용자 방송은 기본값)
    let next = create(&app, &owner, "다음 방송").await;
    assert_eq!(get(&app, &format!("{B}/{next}"), None).await.json["overlayLayout"], layout);
    let theirs = create(&app, &other, "남의 방송").await;
    assert!(get(&app, &format!("{B}/{theirs}"), None).await.json["overlayLayout"].is_null());

    // LIVE에서도 수정 가능, ENDED는 잠김
    start_live(&state.db, id).await.unwrap();
    let moved = json!({
        "alert": { "x": 50.0, "y": 4.0 },
        "cam": { "x": 76.0, "y": 75.0, "w": 22.0 },
        "chat": { "x": 2.0, "y": 94.0, "w": 30.0 }
    });
    let res = patch(&app, &owner, id, json!({ "overlayLayout": moved })).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    assert_eq!(res.json["overlayLayout"], moved);
    end_live(&state, id, EndReason::Owner).await.unwrap();
    let res = patch(&app, &owner, id, json!({ "overlayLayout": moved })).await;
    assert_eq!(
        (res.status, code(&res)),
        (StatusCode::CONFLICT, "BROADCAST_FIELD_LOCKED")
    );
}
