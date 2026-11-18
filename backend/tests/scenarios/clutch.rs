//! 클러치 시나리오 (features/07, api.md §7). 끝에서 지갑 정합성(`mismatched_wallets`)이 비어야 한다.
//! SUPER BURST(T24)·회전·히어로(T29)도 여기 둔다.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::Router;
use axum::http::StatusCode;
use flutch::clutch::{ClutchGauge, ClutchPhase, Outcome};
use flutch::wallet;
use serde_json::{Value, json};
use sqlx::{Connection, PgConnection, PgPool};

use crate::common::{
    FakeLiveKit, ORIGIN, TestResponse, empty_req, json_req, login_as, send, signup, test_app,
    test_state,
};
use crate::realtime::{Ws, connect, next_event};

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
    // 5회 × 10점 = 50점 = 0%, 100%까지 99,950비트
    let d = detail(&app, &viewer, id).await;
    assert_eq!(
        (
            d["gauge"]["percent"].as_i64(),
            d["gauge"]["bitsToFull"].as_i64(),
            d["viewer"]["freeClutchesLeft"].as_i64()
        ),
        (Some(0), Some(99_950), Some(0))
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
    assert_eq!(count(&db, "SELECT count(*) FROM clutch_events WHERE broadcast_id = $1 AND source = 'paid' AND points = 10", id).await, 1);
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

    seed_points(&db, id, viewer_id, 30_000, 301).await; // 5분 밖: 빠진다
    seed_points(&db, id, viewer_id, 20_000, 240).await;
    seed_points(&db, id, viewer_id, 30_000, 5).await;
    let d = detail(&app, &viewer, id).await;
    assert_eq!(
        (d["gauge"]["percent"].clone(), d["gauge"]["phase"].clone()),
        (json!(50), json!("charging"))
    );

    seed_points(&db, id, viewer_id, 30_000, 1).await; // 80,000점 = 80% 임박
    let d = detail(&app, &viewer, id).await;
    assert_eq!(
        (d["gauge"]["percent"].clone(), d["gauge"]["phase"].clone()),
        (json!(80), json!("imminent"))
    );

    // 4분 전 점수까지 5분 밖으로 밀려나면 줄어든다
    sqlx::query("UPDATE clutch_events SET created_at = created_at - interval '70 seconds' WHERE points = 20000")
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

    // 99,990점 + 동시 4명(각 10점): 모두 100%를 넘길 수 있지만 발동은 한 번이다
    seed_points(&db, id, viewer_id, 99_990, 10).await;
    let rs = futures_join(&app, &cookies, id).await;
    assert!(
        rs.iter().all(|r| r.status == StatusCode::OK),
        "{:?}",
        rs.iter().map(|r| r.json.clone()).collect::<Vec<_>>()
    );
    let supers = "SELECT count(*) FROM super_clutches WHERE broadcast_id = $1";
    assert_eq!(count(&db, supers, id).await, 1);
    assert_eq!(count(&db, "SELECT count(*) FROM super_clutches WHERE broadcast_id = $1 AND expires_at = triggered_at + interval '5 minutes'", id).await, 1);

    // 발동 뒤 게이지는 0에서 다시 시작(나머지 3명 30점), 쿨다운
    let d = detail(&app, &viewer, id).await;
    assert_eq!(
        (
            d["gauge"]["percent"].clone(),
            d["gauge"]["phase"].clone(),
            d["gauge"]["bitsToFull"].clone()
        ),
        (json!(0), json!("cooldown"), json!(99_970))
    );
    assert!(d["gauge"]["cooldownUntil"].is_string());
    assert!(d["superClutchUntil"].is_string());

    // 쿨다운 중에는 100%가 돼도 재발동 없이 게이지만 오른다
    seed_points(&db, id, viewer_id, 100_000, 0).await;
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

    seed_points(&db, id, viewer_id, 99_990, 10).await;
    assert_eq!(clutch(&app, Some(&viewer), id).await.status, StatusCode::OK);
    let ev = next_event(&mut ws).await;
    assert_eq!(ev["type"], "clutch.super");
    assert!(ev["data"]["triggeredAt"].is_string() && ev["data"]["expiresAt"].is_string());
}

// ---- SUPER BURST (T24) ----

/// REST 앱 + 같은 상태의 WebSocket 서버 주소.
async fn serve(db: &PgPool) -> (Router, SocketAddr) {
    let state = test_state(db.clone(), Arc::new(FakeLiveKit::default()));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = flutch::app(state.clone());
    tokio::spawn(async move { axum::serve(listener, server).await.unwrap() });
    (flutch::app(state), addr)
}

async fn burst(
    app: &Router,
    cookie: Option<&str>,
    id: i64,
    amount: i64,
    anonymous: bool,
) -> TestResponse {
    send(
        app,
        json_req(
            "POST",
            &format!("{B}/{id}/bursts"),
            cookie,
            json!({ "amount": amount, "anonymous": anonymous }),
        ),
    )
    .await
}

fn code(res: &TestResponse) -> (StatusCode, &str) {
    (res.status, res.json["error"]["code"].as_str().unwrap_or(""))
}

/// 다음 `ty` 이벤트(다른 이벤트는 건너뛴다).
async fn next_of(ws: &mut Ws, ty: &str) -> Value {
    loop {
        let ev = next_event(ws).await;
        if ev["type"] == ty {
            return ev;
        }
    }
}

async fn balance(db: &PgPool, user_id: i64) -> i64 {
    count(
        db,
        "SELECT COALESCE((SELECT balance FROM wallets WHERE user_id = $1), 0)",
        user_id,
    )
    .await
}

#[sqlx::test]
async fn burst_amount_raises_gauge_by_bits(db: PgPool) {
    let app = test_app(db.clone());
    let (owner, _) = login_as(&app, "streamer1").await;
    let (viewer, viewer_id) = login_as(&app, "viewer").await;
    let id = live_broadcast(&app, &owner).await;
    let d = detail(&app, &viewer, id).await;
    assert_eq!(
        (d["burstMin"].clone(), d["gauge"]["bitsToFull"].clone()),
        (json!(1000), json!(100_000))
    );
    charge(&db, viewer_id, 6000).await;

    // 최소 미만·상한 초과는 422 fields.amount, 아무것도 남지 않는다
    for amount in [999, 1_000_001] {
        let res = burst(&app, Some(&viewer), id, amount, false).await;
        assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY, "{}", res.json);
        assert!(
            res.json["error"]["fields"]["amount"].is_string(),
            "{}",
            res.json
        );
    }
    assert_eq!(balance(&db, viewer_id).await, 6000);

    // 5,000비트 = 5,000점 = 5%, 남은 95,000비트
    let res = burst(&app, Some(&viewer), id, 5000, false).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    let d = &res.json["donation"];
    assert_eq!(
        (
            d["kind"].clone(),
            d["amount"].clone(),
            d["donor"]["id"].clone()
        ),
        (json!("burst"), json!(5000), json!(viewer_id))
    );
    let g = &res.json["gauge"];
    assert_eq!(
        (
            res.json["balance"].clone(),
            g["percent"].clone(),
            g["phase"].clone(),
            g["bitsToFull"].clone()
        ),
        (json!(1000), json!(5), json!("charging"), json!(95_000))
    );
    assert_eq!(count(&db, "SELECT count(*) FROM ledger_entries WHERE broadcast_id = $1 AND type = 'burst' AND amount = -5000", id).await, 1);
    assert_eq!(count(&db, "SELECT count(*) FROM donations d JOIN ledger_entries l ON l.id = d.ledger_entry_id WHERE d.broadcast_id = $1 AND d.kind = 'burst' AND d.amount = 5000 AND NOT d.anonymous", id).await, 1);
    assert_eq!(count(&db, "SELECT count(*) FROM clutch_events WHERE broadcast_id = $1 AND source = 'burst' AND points = 5000", id).await, 1);

    // 잔액 부족: 409, 아무것도 남지 않는다
    let res = burst(&app, Some(&viewer), id, 2000, false).await;
    assert_eq!(
        code(&res),
        (StatusCode::CONFLICT, "WALLET_INSUFFICIENT_BALANCE")
    );
    assert_eq!(balance(&db, viewer_id).await, 1000);
    let donations = "SELECT count(*) FROM donations WHERE broadcast_id = $1";
    assert_eq!(count(&db, donations, id).await, 1);
    let events = "SELECT count(*) FROM clutch_events WHERE broadcast_id = $1";
    assert_eq!(count(&db, events, id).await, 1);
    assert_consistent(&db).await;
}

#[sqlx::test]
async fn burst_fill_to_full_triggers_super_clutch(db: PgPool) {
    let app = test_app(db.clone());
    let (owner, _) = login_as(&app, "streamer1").await;
    let (viewer, viewer_id) = login_as(&app, "viewer").await;
    let id = live_broadcast(&app, &owner).await;
    charge(&db, viewer_id, 100_000).await;
    for _ in 0..3 {
        assert_eq!(clutch(&app, Some(&viewer), id).await.status, StatusCode::OK);
    }
    let bits_to_full = |v: &Value| v["gauge"]["bitsToFull"].as_i64().unwrap();
    assert_eq!(bits_to_full(&detail(&app, &viewer, id).await), 99_970);

    let res = burst(&app, Some(&viewer), id, 1000, false).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    let left = bits_to_full(&res.json);
    assert_eq!(left, 98_970);
    let supers = "SELECT count(*) FROM super_clutches WHERE broadcast_id = $1";
    assert_eq!(count(&db, supers, id).await, 0);

    // 남은 비트만큼 버스트하면 정확히 100% → 발동(게이지 0, 쿨다운, 다시 100,000 남음)
    let res = burst(&app, Some(&viewer), id, left, true).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    let g = &res.json["gauge"];
    assert_eq!(
        (
            res.json["balance"].clone(),
            g["percent"].clone(),
            g["phase"].clone(),
            g["bitsToFull"].clone()
        ),
        (json!(30), json!(0), json!("cooldown"), json!(100_000))
    );
    assert_eq!(count(&db, supers, id).await, 1);
    assert_consistent(&db).await;
}

#[sqlx::test]
async fn burst_anonymous_hidden_from_public(db: PgPool) {
    let (app, addr) = serve(&db).await;
    let (owner, _) = login_as(&app, "streamer1").await;
    let (viewer, viewer_id) = login_as(&app, "viewer").await;
    let id = live_broadcast(&app, &owner).await;
    charge(&db, viewer_id, 1000).await;
    let mut public_ws = connect(addr, id, Some(ORIGIN), None).await.unwrap();
    let mut owner_ws = connect(addr, id, Some(ORIGIN), Some(&owner)).await.unwrap();

    let res = burst(&app, Some(&viewer), id, 1000, true).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    assert_eq!(res.json["donation"]["donor"]["id"], viewer_id); // 내 버스트는 실명
    assert_eq!(res.json["donation"]["anonymous"], true);

    let ev = next_of(&mut public_ws, "donation.created").await;
    assert_eq!(ev["data"]["donation"]["kind"], "burst");
    assert_eq!(ev["data"]["donation"]["donor"], Value::Null);
    assert_eq!(ev["data"]["total"], 1000);
    let ev = next_of(&mut owner_ws, "donation.created").await;
    assert_eq!(ev["data"]["donation"]["donor"]["id"], viewer_id);
    // 1,000비트는 이제 1%: 발동 없이 게이지만 오른다
    let ev = next_of(&mut public_ws, "clutch.gauge").await;
    assert_eq!(ev["data"]["bitsToFull"], 99_000);

    let list = |cookie: Option<String>| {
        let app = app.clone();
        async move {
            let path = format!("{B}/{id}/donations");
            let res = send(&app, empty_req("GET", &path, cookie.as_deref())).await;
            assert_eq!(res.status, StatusCode::OK, "{}", res.json);
            res.json["items"][0]["donor"].clone()
        }
    };
    assert_eq!(list(None).await, Value::Null);
    assert_eq!(list(Some(viewer.clone())).await, Value::Null);
    assert_eq!(list(Some(owner.clone())).await["id"], viewer_id);
    assert_consistent(&db).await;
}

#[sqlx::test]
async fn burst_rules(db: PgPool) {
    let app = test_app(db.clone());
    let (owner, owner_id) = login_as(&app, "streamer1").await;
    let (viewer, viewer_id) = login_as(&app, "viewer").await;
    let id = live_broadcast(&app, &owner).await;
    charge(&db, viewer_id, 5000).await;
    charge(&db, owner_id, 5000).await;

    assert_eq!(
        burst(&app, None, id, 1000, false).await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        burst(&app, Some(&viewer), 999_999, 1000, false)
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    let res = burst(&app, Some(&owner), id, 1000, false).await;
    assert_eq!(code(&res), (StatusCode::FORBIDDEN, "CLUTCH_SELF"));

    // READY 방송 409
    let body = json!({ "title": "준비중", "category": "game" });
    let res = send(&app, json_req("POST", B, Some(&owner), body)).await;
    let ready = res.json["id"].as_i64().unwrap();
    let res = burst(&app, Some(&viewer), ready, 1000, false).await;
    assert_eq!(code(&res), (StatusCode::CONFLICT, "BROADCAST_NOT_LIVE"));

    // 잔액 0인 사용자 409
    let poor = signup(&app, "poorburst", "빈지갑").await;
    let res = burst(&app, Some(&poor), id, 1000, false).await;
    assert_eq!(
        code(&res),
        (StatusCode::CONFLICT, "WALLET_INSUFFICIENT_BALANCE")
    );

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
    ban(None).await;
    let res = burst(&app, Some(&viewer), id, 1000, false).await;
    assert_eq!(code(&res), (StatusCode::FORBIDDEN, "MODERATION_BANNED"));
    // 실패한 요청은 원장·후원·게이지에 아무것도 남기지 않는다
    assert_eq!(balance(&db, viewer_id).await, 5000);
    let donations = "SELECT count(*) FROM donations WHERE donor_id = $1";
    assert_eq!(count(&db, donations, viewer_id).await, 0);
    let events = "SELECT count(*) FROM clutch_events WHERE user_id = $1";
    assert_eq!(count(&db, events, viewer_id).await, 0);

    // 임시차단은 허용
    ban(Some(600)).await;
    let res = burst(&app, Some(&viewer), id, 1000, false).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    assert_eq!(balance(&db, viewer_id).await, 4000);
    assert_consistent(&db).await;
}

#[sqlx::test]
async fn burst_concurrent_only_balance_succeeds(db: PgPool) {
    let app = test_app(db.clone());
    let (owner, _) = login_as(&app, "streamer1").await;
    let (viewer, viewer_id) = login_as(&app, "viewer").await;
    let id = live_broadcast(&app, &owner).await;
    charge(&db, viewer_id, 1000).await;

    // 지갑 행을 잠가 두 요청이 모두 차감 UPDATE에서 기다리게 한 뒤(=실제로 겹침) 함께 풀어 준다.
    // 잠금·확인은 테스트 풀 밖의 별도 커넥션으로(T22 트러블슈팅).
    let mut locker = PgConnection::connect_with(&db.connect_options())
        .await
        .unwrap();
    sqlx::query("BEGIN").execute(&mut locker).await.unwrap();
    sqlx::query("SELECT balance FROM wallets WHERE user_id = $1 FOR UPDATE")
        .bind(viewer_id)
        .execute(&mut locker)
        .await
        .unwrap();
    let release = async {
        let mut waited = 0;
        loop {
            sqlx::query("SELECT pg_stat_clear_snapshot()")
                .execute(&mut locker)
                .await
                .unwrap();
            let n: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM pg_stat_activity
                 WHERE datname = current_database() AND wait_event_type = 'Lock'",
            )
            .fetch_one(&mut locker)
            .await
            .unwrap();
            if n == 2 {
                break;
            }
            waited += 1;
            assert!(waited < 1200, "두 요청이 잠금에서 겹치지 않음");
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        sqlx::query("COMMIT").execute(&mut locker).await.unwrap();
    };
    let (r1, r2, ()) = tokio::join!(
        burst(&app, Some(&viewer), id, 1000, false),
        burst(&app, Some(&viewer), id, 1000, true),
        release
    );
    let mut results = [code(&r1), code(&r2)];
    results.sort();
    assert_eq!(
        results,
        [
            (StatusCode::OK, ""),
            (StatusCode::CONFLICT, "WALLET_INSUFFICIENT_BALANCE")
        ],
        "{} {}",
        r1.json,
        r2.json
    );
    assert_eq!(balance(&db, viewer_id).await, 0);
    let donations = "SELECT count(*) FROM donations WHERE donor_id = $1";
    assert_eq!(count(&db, donations, viewer_id).await, 1);
    let events = "SELECT count(*) FROM clutch_events WHERE broadcast_id = $1";
    assert_eq!(count(&db, events, id).await, 1);
    let ledger = "SELECT count(*) FROM ledger_entries WHERE user_id = $1";
    assert_eq!(count(&db, ledger, viewer_id).await, 2); // 충전 1 + 버스트 1
    assert_consistent(&db).await;
}

/// 게이지 이벤트는 방송당 0.5초에 한 번, 간격 안의 것은 마지막 값만. 묶음 상태는 앱 인스턴스(AppState)마다 따로다.
#[sqlx::test]
async fn clutch_gauge_event_throttled(db: PgPool) {
    let state = test_state(db.clone(), Arc::new(FakeLiveKit::default()));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = flutch::app(state.clone());
    tokio::spawn(async move { axum::serve(listener, server).await.unwrap() });
    let app = flutch::app(state.clone());
    let (other, other_addr) = serve(&db).await;
    let (owner, _) = login_as(&app, "streamer1").await;
    let (viewer, _) = login_as(&app, "viewer").await;
    let id = live_broadcast(&app, &owner).await;
    let mut ws = connect(addr, id, Some(ORIGIN), None).await.unwrap();
    let mut other_ws = connect(other_addr, id, Some(ORIGIN), None).await.unwrap();

    // 클러치(실제 경로) → 바로 보낸다. 곧이어 간격 안의 게이지 두 개는 묶여 마지막 값만 0.5초 뒤에.
    let t0 = Instant::now();
    assert_eq!(clutch(&app, Some(&viewer), id).await.status, StatusCode::OK);
    for percent in [20, 30] {
        let o = Outcome {
            gauge: ClutchGauge {
                percent,
                phase: ClutchPhase::Charging,
                cooldown_until: None,
                bits_to_full: 0,
            },
            super_clutch: None,
        };
        flutch::clutch::publish(&state, id, &o);
    }

    // 다른 앱 인스턴스는 묶음을 공유하지 않는다: 같은 방송이어도 바로 보낸다
    // (전역 static이면 위 묶음에 막혀 이 값이 첫 앱의 연결로 나간다)
    assert_eq!(
        clutch(&other, Some(&viewer), id).await.status,
        StatusCode::OK
    );
    // 두 앱의 클러치 2번(각 10점) = 20점
    assert_eq!(
        next_of(&mut other_ws, "clutch.gauge").await["data"]["bitsToFull"],
        99_980
    );

    assert_eq!(next_of(&mut ws, "clutch.gauge").await["data"]["percent"], 0);
    let second = next_of(&mut ws, "clutch.gauge").await;
    assert_eq!(second["data"]["percent"], 30);
    assert!(
        t0.elapsed() >= Duration::from_millis(490),
        "{:?}",
        t0.elapsed()
    );
    let more = tokio::time::timeout(Duration::from_millis(700), next_of(&mut ws, "clutch.gauge"));
    assert!(more.await.is_err(), "묶인 값은 한 번만 보낸다");
}

// ---- SUPER CLUTCH 회전·홈 히어로 (T29) ----

async fn hero(app: &Router) -> Value {
    let res = send(app, empty_req("GET", "/api/v1/home/hero", None)).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    res.json
}

/// 진행 중 목록의 방송 id(회전 순서)
fn active_ids(h: &Value) -> Vec<i64> {
    h["active"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["broadcast"]["id"].as_i64().unwrap())
        .collect()
}

fn ts(v: &Value) -> chrono::DateTime<chrono::FixedOffset> {
    chrono::DateTime::parse_from_rfc3339(v.as_str().unwrap()).unwrap()
}

/// 10만 점 + 클러치 1번으로 SUPER CLUTCH 발동
async fn trigger(app: &Router, db: &PgPool, viewer: (&str, i64), id: i64) {
    seed_points(db, id, viewer.1, 100_000, 1).await;
    let res = clutch(app, Some(viewer.0), id).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    assert_eq!(res.json["gauge"]["phase"], "cooldown", "발동해야 한다");
}

/// 시간 조작: 저장된 발동·노출·점수 시각을 `secs`초 과거로 민다(= `secs`초가 흐른 것과 같다).
async fn travel(db: &PgPool, secs: i64) {
    sqlx::query(
        "UPDATE super_clutches SET triggered_at = triggered_at - $1 * interval '1 second',
                expires_at = expires_at - $1 * interval '1 second',
                shown_at = shown_at - $1 * interval '1 second'",
    )
    .bind(secs as f64)
    .execute(db)
    .await
    .unwrap();
    sqlx::query("UPDATE clutch_events SET created_at = created_at - $1 * interval '1 second'")
        .bind(secs as f64)
        .execute(db)
        .await
        .unwrap();
}

#[sqlx::test]
async fn clutch_rotation(db: PgPool) {
    let state = test_state(db.clone(), Arc::new(FakeLiveKit::default()));
    let app = flutch::app(state.clone());
    let (viewer, viewer_id) = login_as(&app, "viewer").await;
    let v = (viewer.as_str(), viewer_id);
    let a = live_broadcast(&app, &login_as(&app, "streamer1").await.0).await;
    let b = live_broadcast(&app, &login_as(&app, "streamer2").await.0).await;
    let c_owner = signup(&app, "rotator", "회전러").await;
    let c = live_broadcast(&app, &c_owner).await;

    // 발동 2개: 먼저 터진 a가 30초 슬롯으로 노출, 다음은 b
    trigger(&app, &db, v, a).await;
    trigger(&app, &db, v, b).await;
    let h = hero(&app).await;
    assert_eq!(h["mode"], "super_clutch");
    assert_eq!(h["broadcast"]["id"], a);
    assert_eq!(active_ids(&h), [a, b]);
    let slot = &h["slot"];
    assert_eq!(
        (ts(&slot["nextSwitchAt"]) - ts(&slot["startedAt"])).num_seconds(),
        30
    );

    // 30초 뒤: 1초 작업이 DB에서 b로 넘긴다
    travel(&db, 30).await;
    flutch::clutch::rotate(state.clone()).await.unwrap();
    let shown: i64 = sqlx::query_scalar(
        "SELECT broadcast_id FROM super_clutches ORDER BY shown_at DESC NULLS LAST LIMIT 1",
    )
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(shown, b);
    assert_eq!(active_ids(&hero(&app).await), [b, a]);

    // 새 발동 c는 맨 뒤가 아니라 바로 다음 순서
    trigger(&app, &db, v, c).await;
    assert_eq!(active_ids(&hero(&app).await), [b, c, a]);

    // 진행 중인 a가 다시 100%가 돼도 재등록하지 않는다(쿨다운)
    seed_points(&db, a, viewer_id, 100_000, 0).await;
    let res = clutch(&app, Some(&viewer), a).await;
    assert_eq!(res.json["gauge"]["phase"], "cooldown");
    let supers = "SELECT count(*) FROM super_clutches WHERE broadcast_id = $1";
    assert_eq!(count(&db, supers, a).await, 1);
    assert_eq!(active_ids(&hero(&app).await), [b, c, a]);

    travel(&db, 30).await;
    flutch::clutch::rotate(state.clone()).await.unwrap();
    assert_eq!(active_ids(&hero(&app).await), [c, a, b]);

    // 노출 중인 c의 방송이 끝나면 즉시 빠지고 다음(a)이 새 슬롯으로 바로 나온다
    let res = send(
        &app,
        empty_req("DELETE", &format!("{B}/{c}/live"), Some(&c_owner)),
    )
    .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    let h = hero(&app).await;
    assert_eq!(active_ids(&h), [a, b]);
    let age = chrono::Utc::now().fixed_offset() - ts(&h["slot"]["startedAt"]);
    assert!(age.num_seconds() < 5, "새 슬롯이어야 한다: {age}");

    // 발동 5분이 지나면 회전에서 빠지고, 게이지가 모두 0이라 시청자 수 1위로
    travel(&db, 300).await;
    flutch::clutch::rotate(state.clone()).await.unwrap();
    let h = hero(&app).await;
    assert_eq!(
        (h["mode"].clone(), h["slot"].clone()),
        (json!("top_viewers"), Value::Null)
    );
    assert_eq!(active_ids(&h), Vec::<i64>::new());
    assert_eq!(h["broadcast"]["superClutchUntil"], Value::Null);
}

#[sqlx::test]
async fn hero_modes(db: PgPool) {
    let app = test_app(db.clone());
    let (viewer, viewer_id) = login_as(&app, "viewer").await;
    let v = (viewer.as_str(), viewer_id);

    // LIVE가 없으면 empty
    let h = hero(&app).await;
    assert_eq!(
        h,
        json!({ "mode": "empty", "broadcast": null, "gauge": null, "slot": null, "active": [] })
    );

    // 게이지가 모두 0이면 시청자 수 1위(동률은 최신 방송)
    let a = live_broadcast(&app, &login_as(&app, "streamer1").await.0).await;
    let b = live_broadcast(&app, &login_as(&app, "streamer2").await.0).await;
    let h = hero(&app).await;
    assert_eq!(
        (h["mode"].clone(), h["broadcast"]["id"].clone()),
        (json!("top_viewers"), json!(b))
    );
    assert_eq!(h["gauge"]["percent"], 0);

    // 게이지가 있으면 게이지 1위
    seed_points(&db, a, viewer_id, 5_000, 0).await;
    let h = hero(&app).await;
    assert_eq!(
        (
            h["mode"].clone(),
            h["broadcast"]["id"].clone(),
            h["gauge"]["percent"].clone()
        ),
        (json!("top_gauge"), json!(a), json!(5))
    );
    assert_eq!(
        (h["slot"].clone(), h["active"].clone()),
        (Value::Null, json!([]))
    );

    // 진행 중 6개 → super_clutch, 목록은 최대 5개(먼저 터진 순)
    let mut ids = vec![a, b];
    for n in 0..4 {
        let owner = signup(&app, &format!("hero{n}"), &format!("히어로{n}")).await;
        ids.push(live_broadcast(&app, &owner).await);
    }
    for &id in &ids {
        trigger(&app, &db, v, id).await;
    }
    let h = hero(&app).await;
    assert_eq!(h["mode"], "super_clutch");
    assert_eq!(
        (
            h["broadcast"]["id"].as_i64(),
            h["gauge"]["percent"].as_i64()
        ),
        (Some(a), Some(0))
    );
    assert!(h["broadcast"]["superClutchUntil"].is_string());
    assert_eq!(active_ids(&h), ids[..5]);
    assert!(h["active"][0]["expiresAt"].is_string());
    let slot = &h["slot"];
    assert_eq!(
        (ts(&slot["nextSwitchAt"]) - ts(&slot["startedAt"])).num_seconds(),
        30
    );
}

#[sqlx::test]
async fn rotation_survives_restart(db: PgPool) {
    let app = test_app(db.clone());
    let (viewer, viewer_id) = login_as(&app, "viewer").await;
    let v = (viewer.as_str(), viewer_id);
    let a = live_broadcast(&app, &login_as(&app, "streamer1").await.0).await;
    let b = live_broadcast(&app, &login_as(&app, "streamer2").await.0).await;
    trigger(&app, &db, v, a).await;
    trigger(&app, &db, v, b).await;
    let before = hero(&app).await;

    // 서버를 새로 띄워도(새 앱 상태) 같은 슬롯이 이어진다
    let restarted = test_app(db.clone());
    let after = hero(&restarted).await;
    assert_eq!(
        (
            &after["broadcast"]["id"],
            &after["slot"],
            active_ids(&after)
        ),
        (&json!(a), &before["slot"], vec![a, b])
    );
    travel(&db, 30).await;
    assert_eq!(hero(&restarted).await["broadcast"]["id"], b);
}

/// 트랜잭션 중에 풀에서 연결을 하나 더 잡으면(차단 확인) 동시 요청이 풀을 다 써 교착된다(T45 감사).
/// 연결 2개짜리 풀에서 방송 행 잠금으로 두 요청이 트랜잭션 연결을 쥔 채 겹치게 한 뒤 풀어도 셋 다 끝나야 한다.
#[sqlx::test]
async fn burst_concurrent_small_pool_no_deadlock(db: PgPool) {
    let small = sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .acquire_timeout(Duration::from_secs(3))
        .connect_with((*db.connect_options()).clone())
        .await
        .unwrap();
    let app = flutch::app(test_state(small, Arc::new(FakeLiveKit::default())));
    let (owner, _) = login_as(&app, "streamer1").await;
    let (viewer, viewer_id) = login_as(&app, "viewer").await;
    let id = live_broadcast(&app, &owner).await;
    charge(&db, viewer_id, 3000).await;

    let mut locker = PgConnection::connect_with(&db.connect_options())
        .await
        .unwrap();
    sqlx::query("BEGIN").execute(&mut locker).await.unwrap();
    sqlx::query("SELECT 1 FROM broadcasts WHERE id = $1 FOR UPDATE")
        .bind(id)
        .execute(&mut locker)
        .await
        .unwrap();
    let release = async {
        let mut waited = 0;
        loop {
            sqlx::query("SELECT pg_stat_clear_snapshot()")
                .execute(&mut locker)
                .await
                .unwrap();
            let n: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM pg_stat_activity
                 WHERE datname = current_database() AND wait_event_type = 'Lock'",
            )
            .fetch_one(&mut locker)
            .await
            .unwrap();
            if n == 2 {
                break;
            }
            waited += 1;
            assert!(waited < 1200, "두 요청이 잠금에서 겹치지 않음");
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        sqlx::query("COMMIT").execute(&mut locker).await.unwrap();
    };
    let (r1, r2, r3, ()) = tokio::join!(
        burst(&app, Some(&viewer), id, 1000, false),
        burst(&app, Some(&viewer), id, 1000, false),
        burst(&app, Some(&viewer), id, 1000, false),
        release
    );
    for r in [&r1, &r2, &r3] {
        assert_eq!(r.status, StatusCode::OK, "{}", r.json);
    }
    assert_eq!(balance(&db, viewer_id).await, 0);
    assert_consistent(&db).await;
}
