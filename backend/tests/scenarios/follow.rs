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
