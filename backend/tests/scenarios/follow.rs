//! 팔로우·스트리머 페이지 시나리오 (features/09, api.md §9).

use axum::http::StatusCode;
use flutch::broadcast::start_live;
use serde_json::json;
use sqlx::PgPool;

use crate::common::{FakeLiveKit, empty_req, json_req, login_as, send, test_state};
use std::sync::Arc;

#[sqlx::test]
async fn user_profile_public_read(db: PgPool) {
    let state = test_state(db, Arc::new(FakeLiveKit::default()));
    let app = flutch::app(state.clone());
    let (c1, u1) = login_as(&app, "streamer1").await;
    let (cv, uv) = login_as(&app, "viewer").await;
    let path = format!("/api/v1/users/{u1}");

    // 비로그인: following null, 방송 없음
    let res = send(&app, empty_req("GET", &path, None)).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.json["user"]["id"], u1);
    assert_eq!(res.json["followerCount"], 0);
    assert!(res.json["following"].is_null());
    assert!(res.json["liveBroadcast"].is_null());

    // 로그인: following false → 팔로우 후 true, 팔로워 수 1
    let res = send(&app, empty_req("GET", &path, Some(&cv))).await;
    assert_eq!(res.json["following"], false);
    sqlx::query!(
        "INSERT INTO follows (follower_id, streamer_id) VALUES ($1, $2)",
        uv,
        u1
    )
    .execute(&state.db)
    .await
    .unwrap();
    let res = send(&app, empty_req("GET", &path, Some(&cv))).await;
    assert_eq!(res.json["following"], true);
    assert_eq!(res.json["followerCount"], 1);

    // LIVE 방송이 있으면 포함(READY는 제외)
    let body = json!({ "title": "라이브 방송", "category": "game", "tags": [] });
    let res = send(
        &app,
        json_req("POST", "/api/v1/broadcasts", Some(&c1), body),
    )
    .await;
    let bid = res.json["id"].as_i64().unwrap();
    let res = send(&app, empty_req("GET", &path, None)).await;
    assert!(res.json["liveBroadcast"].is_null());
    start_live(&state.db, bid).await.unwrap();
    let res = send(&app, empty_req("GET", &path, None)).await;
    assert_eq!(res.json["liveBroadcast"]["id"], bid);

    // 없는 사용자·탈퇴한 사용자는 404
    let res = send(&app, empty_req("GET", "/api/v1/users/999999", None)).await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
    sqlx::query!("UPDATE users SET withdrawn_at = now() WHERE id = $1", u1)
        .execute(&state.db)
        .await
        .unwrap();
    let res = send(&app, empty_req("GET", &path, None)).await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn follow_toggle_and_count(db: PgPool) {
    let state = test_state(db, Arc::new(FakeLiveKit::default()));
    let app = flutch::app(state.clone());
    let (_, u1) = login_as(&app, "streamer1").await;
    let (cv, _) = login_as(&app, "viewer").await;
    let (c2, u2) = login_as(&app, "streamer2").await;
    let path = format!("/api/v1/users/{u1}/follow");

    let res = send(&app, empty_req("PUT", &path, None)).await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);

    let res = send(&app, empty_req("PUT", &path, Some(&cv))).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.json, json!({ "following": true, "followerCount": 1 }));
    // 중복 팔로우는 그대로 200
    let res = send(&app, empty_req("PUT", &path, Some(&cv))).await;
    assert_eq!(res.json, json!({ "following": true, "followerCount": 1 }));

    // 탈퇴한 팔로워는 세지 않는다
    send(&app, empty_req("PUT", &path, Some(&c2))).await;
    let profile = format!("/api/v1/users/{u1}");
    assert_eq!(
        send(&app, empty_req("GET", &profile, None)).await.json["followerCount"],
        2
    );
    sqlx::query!("UPDATE users SET withdrawn_at = now() WHERE id = $1", u2)
        .execute(&state.db)
        .await
        .unwrap();
    assert_eq!(
        send(&app, empty_req("GET", &profile, None)).await.json["followerCount"],
        1
    );

    let res = send(&app, empty_req("DELETE", &path, Some(&cv))).await;
    assert_eq!(res.json, json!({ "following": false, "followerCount": 0 }));
    let res = send(&app, empty_req("DELETE", &path, Some(&cv))).await;
    assert_eq!(res.status, StatusCode::OK);

    let res = send(
        &app,
        empty_req("PUT", "/api/v1/users/999999/follow", Some(&cv)),
    )
    .await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn follow_self_rejected(db: PgPool) {
    let app = flutch::app(test_state(db, Arc::new(FakeLiveKit::default())));
    let (c1, u1) = login_as(&app, "streamer1").await;
    let res = send(
        &app,
        empty_req("PUT", &format!("/api/v1/users/{u1}/follow"), Some(&c1)),
    )
    .await;
    assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(res.json["error"]["code"], "FOLLOW_SELF");
}

#[sqlx::test]
async fn pulse_live_first_limit_and_withdrawn(db: PgPool) {
    let state = test_state(db, Arc::new(FakeLiveKit::default()));
    let app = flutch::app(state.clone());
    let (c1, u1) = login_as(&app, "streamer1").await;
    let (_, u2) = login_as(&app, "streamer2").await;
    let (cv, _) = login_as(&app, "viewer").await;

    assert_eq!(
        send(&app, empty_req("GET", "/api/v1/pulse", None))
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
    let res = send(&app, empty_req("GET", "/api/v1/pulse", Some(&cv))).await;
    assert_eq!(res.json, json!({ "live": [], "offline": [], "total": 0 }));

    // streamer1: 과거 방송 종료(offline), streamer2: LIVE — 둘 다 팔로우
    let body = json!({ "title": "지난 방송", "category": "game", "tags": [] });
    let res = send(
        &app,
        json_req("POST", "/api/v1/broadcasts", Some(&c1), body),
    )
    .await;
    let old = res.json["id"].as_i64().unwrap();
    sqlx::query!("UPDATE broadcasts SET status = 'ENDED', started_at = now(), ended_at = now() WHERE id = $1", old)
        .execute(&state.db)
        .await
        .unwrap();
    let (c2, _) = login_as(&app, "streamer2").await;
    let body = json!({ "title": "라이브 방송", "category": "game", "tags": [] });
    let res = send(
        &app,
        json_req("POST", "/api/v1/broadcasts", Some(&c2), body),
    )
    .await;
    start_live(&state.db, res.json["id"].as_i64().unwrap())
        .await
        .unwrap();
    for u in [u1, u2] {
        send(
            &app,
            empty_req("PUT", &format!("/api/v1/users/{u}/follow"), Some(&cv)),
        )
        .await;
    }

    let res = send(&app, empty_req("GET", "/api/v1/pulse", Some(&cv))).await;
    assert_eq!(res.json["total"], 2);
    assert_eq!(res.json["live"][0]["owner"]["id"], u2);
    assert_eq!(res.json["offline"][0]["user"]["id"], u1);
    assert!(res.json["offline"][0]["lastEndedAt"].is_string());

    // limit 1이면 live만, 범위 밖은 422
    let res = send(&app, empty_req("GET", "/api/v1/pulse?limit=1", Some(&cv))).await;
    assert_eq!(
        (
            res.json["live"].as_array().unwrap().len(),
            res.json["offline"].as_array().unwrap().len()
        ),
        (1, 0)
    );
    assert_eq!(
        send(&app, empty_req("GET", "/api/v1/pulse?limit=51", Some(&cv)))
            .await
            .status,
        StatusCode::UNPROCESSABLE_ENTITY
    );

    // 탈퇴한 스트리머는 빠진다
    sqlx::query!("UPDATE users SET withdrawn_at = now() WHERE id = $1", u2)
        .execute(&state.db)
        .await
        .unwrap();
    let res = send(&app, empty_req("GET", "/api/v1/pulse", Some(&cv))).await;
    assert_eq!(
        (
            res.json["total"].as_i64(),
            res.json["live"].as_array().unwrap().len()
        ),
        (Some(1), 0)
    );
}
