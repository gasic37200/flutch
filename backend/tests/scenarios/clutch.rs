//! 클러치 시나리오 (features/07, api.md §7). 끝에서 지갑 정합성(`mismatched_wallets`)이 비어야 한다.
//! `clutch_rotation`은 T29(회전 스케줄러)에서 추가한다.

use std::sync::Arc;

use axum::Router;
use axum::http::StatusCode;
use flutch::wallet;
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::common::{
    FakeLiveKit, ORIGIN, TestResponse, empty_req, json_req, login_as, send, signup, test_app,
    test_state,
};
use crate::realtime::{connect, next_event};

const B: &str = "/api/v1/broadcasts";

async fn live_broadcast(app: &Router, cookie: &str) -> i64 {
    let body = json!({ "title": "클러치 테스트", "category": "game" });
    let res = send(app, json_req("POST", B, Some(cookie), body)).await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.json);
    let id = res.json["id"].as_i64().unwrap();
    let res = send(
        app,
        json_req(
            "POST",
            &format!("{B}/{id}/live"),
            Some(cookie),
            json!({ "tabId": "tab-aaaaaaaa" }),
        ),
    )
    .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    id
}

async fn clutch(app: &Router, cookie: Option<&str>, id: i64) -> TestResponse {
    send(
        app,
        empty_req("POST", &format!("{B}/{id}/clutches"), cookie),
    )
    .await
}

async fn detail(app: &Router, cookie: &str, id: i64) -> Value {
    let res = send(app, empty_req("GET", &format!("{B}/{id}"), Some(cookie))).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    res.json
}

async fn charge(db: &PgPool, user_id: i64, amount: i64) {
    let order_id: i64 = sqlx::query_scalar(
        "INSERT INTO payment_orders (order_id, user_id, purpose, amount, expires_at)
         VALUES (gen_random_uuid()::text, $1, 'charge', $2, now() + interval '30 minutes') RETURNING id",
    )
    .bind(user_id)
    .bind(amount)
    .fetch_one(db)
    .await
    .unwrap();
    let mut tx = db.begin().await.unwrap();
    wallet::credit_charge(&mut tx, user_id, amount, order_id)
        .await
        .unwrap();
    tx.commit().await.unwrap();
}

/// 게이지 점수를 직접 넣는다. `age_secs`만큼 과거 시각으로 기록(시간 조작).
async fn seed_points(db: &PgPool, broadcast_id: i64, user_id: i64, points: i64, age_secs: i64) {
    sqlx::query(
        "INSERT INTO clutch_events (broadcast_id, user_id, source, points, created_at)
         VALUES ($1, $2, 'donation', $3, clock_timestamp() - $4::bigint * interval '1 second')",
    )
    .bind(broadcast_id)
    .bind(user_id)
    .bind(points)
    .bind(age_secs)
    .execute(db)
    .await
    .unwrap();
}

async fn count(db: &PgPool, sql: &'static str, a: i64) -> i64 {
    sqlx::query_scalar(sql).bind(a).fetch_one(db).await.unwrap()
}

async fn assert_consistent(db: &PgPool) {
    assert_eq!(
        wallet::mismatched_wallets(db).await.unwrap(),
        Vec::<i64>::new()
    );
}

#[sqlx::test]
async fn clutch_access_rules(db: PgPool) {
    let app = test_app(db.clone());
    let (owner, _) = login_as(&app, "streamer1").await;
    let (viewer, _) = login_as(&app, "viewer").await;
    let id = live_broadcast(&app, &owner).await;

    assert_eq!(
        clutch(&app, None, id).await.status,
        StatusCode::UNAUTHORIZED
    );
    let res = clutch(&app, Some(&owner), id).await;
    assert_eq!(
        (res.status, &res.json["error"]["code"]),
        (StatusCode::FORBIDDEN, &json!("CLUTCH_SELF"))
    );
    assert_eq!(
        clutch(&app, Some(&viewer), 99999).await.status,
        StatusCode::NOT_FOUND
    );

    // READY 방송은 409
    let res = send(
        &app,
        json_req(
            "POST",
            B,
            Some(&owner),
            json!({ "title": "아직 준비중", "category": "game" }),
        ),
    )
    .await;
    let ready = res.json["id"].as_i64().unwrap();
    let res = clutch(&app, Some(&viewer), ready).await;
    assert_eq!(
        (res.status, &res.json["error"]["code"]),
        (StatusCode::CONFLICT, &json!("BROADCAST_NOT_LIVE"))
    );
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM clutch_events WHERE broadcast_id = $1",
            id
        )
        .await,
        0
    );
    assert_consistent(&db).await;
}

#[sqlx::test]
async fn clutch_free_combo_then_paid(db: PgPool) {
    let app = test_app(db.clone());
    let (owner, _) = login_as(&app, "streamer1").await;
    let (viewer, viewer_id) = login_as(&app, "viewer").await;
    let id = live_broadcast(&app, &owner).await;
    assert_eq!(
        detail(&app, &viewer, id).await["viewer"]["freeClutchesLeft"],
        5
    );

    for n in 1..=5 {
        let res = clutch(&app, Some(&viewer), id).await;
        assert_eq!(res.status, StatusCode::OK, "{}", res.json);
        assert_eq!(res.json["paid"], false);
        assert_eq!(res.json["combo"], n);
        assert_eq!(res.json["freeClutchesLeft"], 5 - n);
        assert_eq!(res.json["balance"], Value::Null);
    }
    // 5회 × 50점 = 250점 = 2%
    let d = detail(&app, &viewer, id).await;
    assert_eq!(
        (
            d["gauge"]["percent"].as_i64(),
            d["viewer"]["freeClutchesLeft"].as_i64()
        ),
        (Some(2), Some(0))
    );

    // 6번째: 잔액 0이면 409, 아무것도 남지 않는다
    let res = clutch(&app, Some(&viewer), id).await;
    assert_eq!(
        (res.status, &res.json["error"]["code"]),
        (StatusCode::CONFLICT, &json!("WALLET_INSUFFICIENT_BALANCE"))
    );
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM clutch_events WHERE broadcast_id = $1",
            id
        )
        .await,
        5
    );
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM donations WHERE broadcast_id = $1",
            id
        )
        .await,
        0
    );

    charge(&db, viewer_id, 1000).await;
    let res = clutch(&app, Some(&viewer), id).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    assert_eq!(
        (
            res.json["paid"].clone(),
            res.json["combo"].clone(),
            res.json["balance"].clone()
        ),
        (json!(true), json!(0), json!(999))
    );
    assert_eq!(count(&db, "SELECT count(*) FROM clutch_events WHERE broadcast_id = $1 AND source = 'paid' AND points = 50", id).await, 1);
    assert_eq!(count(&db, "SELECT count(*) FROM donations WHERE broadcast_id = $1 AND kind = 'clutch' AND amount = 1 AND anonymous", id).await, 1);
    assert_eq!(count(&db, "SELECT count(*) FROM ledger_entries WHERE broadcast_id = $1 AND type = 'clutch' AND amount = -1", id).await, 1);

    // 30초 창이 지나면 무료가 다시 생긴다
    sqlx::query("UPDATE clutch_events SET created_at = created_at - interval '31 seconds'")
        .execute(&db)
        .await
        .unwrap();
    let res = clutch(&app, Some(&viewer), id).await;
    assert_eq!(
        (res.json["paid"].clone(), res.json["combo"].clone()),
        (json!(false), json!(1))
    );
    assert_consistent(&db).await;
}

#[sqlx::test]
async fn clutch_gauge_decay_window(db: PgPool) {
    let app = test_app(db.clone());
    let (owner, _) = login_as(&app, "streamer1").await;
    let (viewer, viewer_id) = login_as(&app, "viewer").await;
    let id = live_broadcast(&app, &owner).await;

    seed_points(&db, id, viewer_id, 3_000, 301).await; // 5분 밖: 빠진다
    seed_points(&db, id, viewer_id, 2_000, 240).await;
    seed_points(&db, id, viewer_id, 3_000, 5).await;
    let d = detail(&app, &viewer, id).await;
    assert_eq!(
        (d["gauge"]["percent"].clone(), d["gauge"]["phase"].clone()),
        (json!(50), json!("charging"))
    );

    seed_points(&db, id, viewer_id, 3_000, 1).await; // 8,000점 = 80% 임박
    let d = detail(&app, &viewer, id).await;
    assert_eq!(
        (d["gauge"]["percent"].clone(), d["gauge"]["phase"].clone()),
        (json!(80), json!("imminent"))
    );

    // 4분 전 점수까지 5분 밖으로 밀려나면 줄어든다
    sqlx::query("UPDATE clutch_events SET created_at = created_at - interval '70 seconds' WHERE points = 2000")
        .execute(&db)
        .await
        .unwrap();
    let d = detail(&app, &viewer, id).await;
    assert_eq!(d["gauge"]["percent"], 60);
}

#[sqlx::test]
async fn super_clutch_trigger_reset_and_cooldown(db: PgPool) {
    let app = test_app(db.clone());
    let (owner, _) = login_as(&app, "streamer1").await;
    let (viewer, viewer_id) = login_as(&app, "viewer").await;
    let id = live_broadcast(&app, &owner).await;
    let mut cookies = vec![viewer.clone()];
    for n in 0..3 {
        cookies.push(signup(&app, &format!("clutcher{n}"), &format!("클러처{n}")).await);
    }

    // 9,950점 + 동시 4명(각 50점): 모두 100%를 넘길 수 있지만 발동은 한 번이다
    seed_points(&db, id, viewer_id, 9_950, 10).await;
    let rs = futures_join(&app, &cookies, id).await;
    assert!(
        rs.iter().all(|r| r.status == StatusCode::OK),
        "{:?}",
        rs.iter().map(|r| r.json.clone()).collect::<Vec<_>>()
    );
    let supers = "SELECT count(*) FROM super_clutches WHERE broadcast_id = $1";
    assert_eq!(count(&db, supers, id).await, 1);
    assert_eq!(count(&db, "SELECT count(*) FROM super_clutches WHERE broadcast_id = $1 AND expires_at = triggered_at + interval '5 minutes'", id).await, 1);

    // 발동 뒤 게이지는 0에서 다시 시작(나머지 3명 150점 = 1%), 쿨다운
    let d = detail(&app, &viewer, id).await;
    assert_eq!(
        (d["gauge"]["percent"].clone(), d["gauge"]["phase"].clone()),
        (json!(1), json!("cooldown"))
    );
    assert!(d["gauge"]["cooldownUntil"].is_string());
    assert!(d["superClutchUntil"].is_string());

    // 쿨다운 중에는 100%가 돼도 재발동 없이 게이지만 오른다
    seed_points(&db, id, viewer_id, 10_000, 0).await;
    let res = clutch(&app, Some(&viewer), id).await;
    assert_eq!(
        (
            res.json["gauge"]["percent"].clone(),
            res.json["gauge"]["phase"].clone()
        ),
        (json!(100), json!("cooldown"))
    );
    assert_eq!(count(&db, supers, id).await, 1);

    // 발동 후 31분: 쿨다운이 끝났으니 다시 발동하고 회전 순서는 뒤로 붙는다
    sqlx::query("UPDATE super_clutches SET triggered_at = triggered_at - interval '31 minutes', expires_at = expires_at - interval '31 minutes'")
        .execute(&db)
        .await
        .unwrap();
    let res = clutch(&app, Some(&viewer), id).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    assert_eq!(res.json["gauge"]["percent"], 0);
    assert_eq!(count(&db, supers, id).await, 2);
    assert_eq!(
        count(
            &db,
            "SELECT MAX(rotation_order) FROM super_clutches WHERE broadcast_id = $1",
            id
        )
        .await,
        2
    );
    assert_consistent(&db).await;
}

async fn futures_join(app: &Router, cookies: &[String], id: i64) -> Vec<TestResponse> {
    let tasks: Vec<_> = cookies
        .iter()
        .map(|c| {
            let (app, c) = (app.clone(), c.clone());
            tokio::spawn(async move { clutch(&app, Some(&c), id).await })
        })
        .collect();
    let mut out = Vec::new();
    for t in tasks {
        out.push(t.await.unwrap());
    }
    out
}

#[sqlx::test]
async fn clutch_banned_rules(db: PgPool) {
    let app = test_app(db.clone());
    let (owner, owner_id) = login_as(&app, "streamer1").await;
    let (viewer, viewer_id) = login_as(&app, "viewer").await;
    let id = live_broadcast(&app, &owner).await;
    charge(&db, viewer_id, 1000).await;
    let ban = |secs: Option<i64>| {
        let db = db.clone();
        async move {
            sqlx::query(
                "INSERT INTO bans (streamer_id, user_id, until)
                 VALUES ($1, $2, now() + $3::bigint * interval '1 second')
                 ON CONFLICT (streamer_id, user_id) DO UPDATE SET until = EXCLUDED.until",
            )
            .bind(owner_id)
            .bind(viewer_id)
            .bind(secs)
            .execute(&db)
            .await
            .unwrap();
        }
    };

    // 영구 밴: 무료는 허용, 5회 뒤 유료는 403
    ban(None).await;
    for _ in 0..5 {
        assert_eq!(clutch(&app, Some(&viewer), id).await.status, StatusCode::OK);
    }
    let res = clutch(&app, Some(&viewer), id).await;
    assert_eq!(
        (res.status, &res.json["error"]["code"]),
        (StatusCode::FORBIDDEN, &json!("MODERATION_BANNED"))
    );
    assert_eq!(
        count(
            &db,
            "SELECT count(*) FROM ledger_entries WHERE user_id = $1 AND type = 'clutch'",
            viewer_id
        )
        .await,
        0
    );

    // 임시차단: 유료도 허용
    ban(Some(600)).await;
    let res = clutch(&app, Some(&viewer), id).await;
    assert_eq!(
        (res.status, res.json["paid"].clone()),
        (StatusCode::OK, json!(true))
    );
    assert_consistent(&db).await;
}

#[sqlx::test]
async fn clutch_super_event_published(db: PgPool) {
    let state = test_state(db.clone(), Arc::new(FakeLiveKit::default()));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = flutch::app(state.clone());
    tokio::spawn(async move { axum::serve(listener, server).await.unwrap() });
    let app = flutch::app(state);
    let (owner, _) = login_as(&app, "streamer1").await;
    let (viewer, viewer_id) = login_as(&app, "viewer").await;
    let id = live_broadcast(&app, &owner).await;
    let mut ws = connect(addr, id, Some(ORIGIN), None).await.unwrap();

    seed_points(&db, id, viewer_id, 9_950, 10).await;
    assert_eq!(clutch(&app, Some(&viewer), id).await.status, StatusCode::OK);
    let ev = next_event(&mut ws).await;
    assert_eq!(ev["type"], "clutch.super");
    assert!(ev["data"]["triggeredAt"].is_string() && ev["data"]["expiresAt"].is_string());
}
