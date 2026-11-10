//! 잔액 후원·후원 목록·스킵·받은 후원 통계·알림 이벤트(docs/features/06, api.md §6·§11).
//! 모든 시나리오는 끝에서 `mismatched_wallets`(잔액 = 마지막 balance_after = 원장 합계)가 비어야 한다.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::http::StatusCode;
use chrono::{Days, NaiveTime, TimeDelta, Utc};
use flutch::wallet;
use serde_json::{Value, json};
use sqlx::{Connection, PgConnection, PgPool};

use crate::common::{
    FakeLiveKit, FakeToss, ORIGIN, TestResponse, empty_req, json_req, login_as, send, signup,
    test_state_with_toss,
};
use crate::realtime::{Ws, connect, next_event};

const B: &str = "/api/v1/broadcasts";

/// REST 앱 + WebSocket 서버 주소(같은 상태).
async fn setup(db: &PgPool) -> (Router, SocketAddr) {
    let state = test_state_with_toss(
        db.clone(),
        Arc::new(FakeLiveKit::default()),
        Arc::new(FakeToss::default()),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = flutch::app(state.clone());
    tokio::spawn(async move { axum::serve(listener, server).await.unwrap() });
    (flutch::app(state), addr)
}

async fn assert_consistent(db: &PgPool) {
    assert_eq!(
        wallet::mismatched_wallets(db).await.unwrap(),
        Vec::<i64>::new()
    );
}

async fn scalar(db: &PgPool, sql: &'static str, id: i64) -> i64 {
    sqlx::query_scalar(sql)
        .bind(id)
        .fetch_one(db)
        .await
        .unwrap()
}

async fn balance(db: &PgPool, user_id: i64) -> i64 {
    scalar(
        db,
        "SELECT COALESCE((SELECT balance FROM wallets WHERE user_id = $1), 0)",
        user_id,
    )
    .await
}

async fn donation_count(db: &PgPool, user_id: i64) -> i64 {
    scalar(
        db,
        "SELECT count(*) FROM donations WHERE donor_id = $1",
        user_id,
    )
    .await
}

/// 충전 주문을 만들어 바로 적립한다.
async fn charge(db: &PgPool, user_id: i64, amount: i64) {
    let order: i64 = sqlx::query_scalar(
        "INSERT INTO payment_orders (order_id, user_id, purpose, amount, expires_at)
         VALUES (gen_random_uuid()::text, $1, 'charge', $2, now() + interval '30 minutes') RETURNING id",
    )
    .bind(user_id)
    .bind(amount)
    .fetch_one(db)
    .await
    .unwrap();
    let mut conn = db.acquire().await.unwrap();
    wallet::credit_charge(&mut conn, user_id, amount, order)
        .await
        .unwrap();
}

async fn create(app: &Router, cookie: &str) -> i64 {
    let body = json!({ "title": "후원 방송", "category": "game" });
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

async fn live_broadcast(app: &Router, cookie: &str) -> i64 {
    let id = create(app, cookie).await;
    go_live(app, cookie, id).await;
    id
}

async fn donate(app: &Router, cookie: Option<&str>, b: i64, body: Value) -> TestResponse {
    send(
        app,
        json_req("POST", &format!("{B}/{b}/donations"), cookie, body),
    )
    .await
}

fn gift(amount: i64) -> Value {
    json!({ "amount": amount, "anonymous": false })
}

fn code(res: &TestResponse) -> (StatusCode, &str) {
    (res.status, res.json["error"]["code"].as_str().unwrap_or(""))
}

async fn list(app: &Router, cookie: Option<&str>, b: i64, q: &str) -> TestResponse {
    send(
        app,
        empty_req("GET", &format!("{B}/{b}/donations{q}"), cookie),
    )
    .await
}

/// 다음 donation.* 이벤트(다른 이벤트는 건너뛴다).
async fn next_donation_event(ws: &mut Ws) -> Value {
    loop {
        let ev = next_event(ws).await;
        if ev["type"].as_str().unwrap().starts_with("donation.") {
            return ev;
        }
    }
}

#[sqlx::test]
async fn donation_live_only(db: PgPool) {
    let (app, _) = setup(&db).await;
    let (owner, _) = login_as(&app, "streamer1").await;
    let (viewer, viewer_id) = login_as(&app, "viewer").await;
    charge(&db, viewer_id, 10_000).await;

    // 비로그인 401, 없는 방송 404
    assert_eq!(
        donate(&app, None, 1, gift(1000)).await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        donate(&app, Some(&viewer), 999_999, gift(1000))
            .await
            .status,
        StatusCode::NOT_FOUND
    );

    // READY → 409, LIVE → 201, ENDED → 409
    let b = create(&app, &owner).await;
    let res = donate(&app, Some(&viewer), b, gift(1000)).await;
    assert_eq!(code(&res), (StatusCode::CONFLICT, "BROADCAST_NOT_LIVE"));
    go_live(&app, &owner, b).await;
    let res = donate(&app, Some(&viewer), b, gift(1000)).await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.json);
    assert_eq!(res.json["balance"], 9000);
    assert_eq!(res.json["donation"]["amount"], 1000);
    assert_eq!(res.json["donation"]["kind"], "donation");
    assert_eq!(res.json["donation"]["donor"]["id"], viewer_id);
    // 후원 비트 × 10점이 게이지에 들어간다(1,000비트 = 10,000점 = SUPER CLUTCH 발동 → 쿨다운)
    let points: i64 = sqlx::query_scalar(
        "SELECT points FROM clutch_events WHERE broadcast_id = $1 AND source = 'donation'",
    )
    .bind(b)
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(points, 10_000);
    let detail = send(&app, empty_req("GET", &format!("{B}/{b}"), None)).await;
    assert_eq!(detail.json["gauge"]["phase"], "cooldown", "{}", detail.json);
    end_live(&app, &owner, b).await;
    let res = donate(&app, Some(&viewer), b, gift(1000)).await;
    assert_eq!(code(&res), (StatusCode::CONFLICT, "BROADCAST_NOT_LIVE"));

    assert_eq!(balance(&db, viewer_id).await, 9000);
    assert_eq!(donation_count(&db, viewer_id).await, 1);
    assert_consistent(&db).await;
}

#[sqlx::test]
async fn donation_input_rules(db: PgPool) {
    let (app, _) = setup(&db).await;
    let (owner, owner_id) = login_as(&app, "streamer1").await;
    let (viewer, viewer_id) = login_as(&app, "viewer").await;
    let b = live_broadcast(&app, &owner).await;
    charge(&db, owner_id, 5000).await;
    charge(&db, viewer_id, 5000).await;

    // 내 방송 후원 403
    let res = donate(&app, Some(&owner), b, gift(1000)).await;
    assert_eq!(code(&res), (StatusCode::FORBIDDEN, "DONATION_SELF"));
    // 금액 범위·메시지 길이 422, FLUTCH 금지어 422
    for amount in [999, 1_000_001, 0] {
        let res = donate(&app, Some(&viewer), b, gift(amount)).await;
        assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY, "{amount}");
        assert!(res.json["error"]["fields"]["amount"].is_string());
    }
    let long = "가".repeat(101);
    let res = donate(
        &app,
        Some(&viewer),
        b,
        json!({ "amount": 1000, "message": long, "anonymous": false }),
    )
    .await;
    assert!(
        res.json["error"]["fields"]["message"].is_string(),
        "{}",
        res.json
    );
    let res = donate(
        &app,
        Some(&viewer),
        b,
        json!({ "amount": 1000, "message": "토토사이트 홍보", "anonymous": false }),
    )
    .await;
    assert_eq!(
        code(&res),
        (StatusCode::UNPROCESSABLE_ENTITY, "DONATION_BANNED_WORD")
    );
    // 잔액 부족 409(화면은 "충전하고 후원하기"로 전환)
    let res = donate(&app, Some(&viewer), b, gift(5001)).await;
    assert_eq!(
        code(&res),
        (StatusCode::CONFLICT, "WALLET_INSUFFICIENT_BALANCE")
    );

    // 정확히 100자 + 앞뒤 공백·욕설: 공백은 잘리고 응답에선 가려진다
    let msg = format!("  {}개새끼  ", "가".repeat(97));
    let res = donate(
        &app,
        Some(&viewer),
        b,
        json!({ "amount": 5000, "message": msg, "anonymous": false }),
    )
    .await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.json);
    assert_eq!(
        res.json["donation"]["message"],
        format!("{}***", "가".repeat(97))
    );
    assert_eq!(res.json["balance"], 0);
    assert_eq!(donation_count(&db, viewer_id).await, 1);
    assert_eq!(donation_count(&db, owner_id).await, 0);
    assert_consistent(&db).await;
}

#[sqlx::test]
async fn donation_anonymous_name_visibility(db: PgPool) {
    let (app, addr) = setup(&db).await;
    let (owner_c, owner_id) = login_as(&app, "streamer1").await;
    let (viewer_c, viewer_id) = login_as(&app, "viewer").await;
    let (other_c, _) = login_as(&app, "streamer2").await;
    let b = live_broadcast(&app, &owner_c).await;
    charge(&db, viewer_id, 10_000).await;

    let mut owner_ws = connect(addr, b, Some(ORIGIN), Some(&owner_c))
        .await
        .unwrap();
    let mut other_ws = connect(addr, b, Some(ORIGIN), Some(&other_c))
        .await
        .unwrap();
    let mut guest_ws = connect(addr, b, Some(ORIGIN), None).await.unwrap();

    let body = json!({ "amount": 3000, "message": "응원해요", "anonymous": true });
    let res = donate(&app, Some(&viewer_c), b, body).await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.json);
    let id = res.json["donation"]["id"].as_i64().unwrap();
    // 내 후원 응답은 실명
    assert_eq!(res.json["donation"]["anonymous"], true);
    assert_eq!(res.json["donation"]["donor"]["id"], viewer_id);

    // 이벤트: 소유자 연결만 실명, 다른 사용자·비로그인은 donor null
    let ev = next_donation_event(&mut owner_ws).await;
    assert_eq!(ev["type"], "donation.created");
    assert_eq!(ev["data"]["total"], 3000);
    assert_eq!(ev["data"]["donation"]["id"], id);
    assert_eq!(ev["data"]["donation"]["donor"]["id"], viewer_id);
    for ws in [&mut other_ws, &mut guest_ws] {
        let ev = next_donation_event(ws).await;
        assert_eq!(ev["type"], "donation.created");
        assert_eq!(ev["data"]["donation"]["id"], id);
        assert_eq!(ev["data"]["donation"]["anonymous"], true);
        assert!(ev["data"]["donation"]["donor"].is_null(), "{ev}");
        assert_eq!(ev["data"]["donation"]["message"], "응원해요");
        assert!(!ev.to_string().contains("\"viewer\""), "{ev}");
    }

    // 공개 이름 후원은 누구에게나 실명
    let res = donate(&app, Some(&viewer_c), b, gift(1000)).await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.json);
    let ev = next_donation_event(&mut guest_ws).await;
    assert_eq!(ev["data"]["donation"]["donor"]["id"], viewer_id);
    assert_eq!(ev["data"]["total"], 4000);

    // 목록: 비로그인·다른 사용자·후원자 본인은 익명 donor null, 소유자만 실명
    for cookie in [None, Some(other_c.as_str()), Some(viewer_c.as_str())] {
        let res = list(&app, cookie, b, "").await;
        assert_eq!(res.status, StatusCode::OK, "{}", res.json);
        let items = res.json["items"].as_array().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["donor"]["id"], viewer_id);
        assert_eq!(items[1]["id"], id);
        assert!(items[1]["donor"].is_null(), "{}", res.json);
    }
    let res = list(&app, Some(&owner_c), b, "").await;
    assert_eq!(res.json["items"][1]["donor"]["id"], viewer_id);
    assert_eq!(res.json["items"][1]["donor"]["nickname"], "시청자");

    // 후원자 배지(익명 포함): 실제 후원 데이터로 붙는다
    let res = send(
        &app,
        json_req(
            "POST",
            &format!("{B}/{b}/chats"),
            Some(&viewer_c),
            json!({ "body": "후원했어요" }),
        ),
    )
    .await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.json);
    assert_eq!(res.json["badges"], json!(["donor"]));
    assert_ne!(owner_id, viewer_id);
    assert_consistent(&db).await;
}

#[sqlx::test]
async fn donation_balance_conditional_debit(db: PgPool) {
    let (app, _) = setup(&db).await;
    let (owner, _) = login_as(&app, "streamer1").await;
    let (viewer, viewer_id) = login_as(&app, "viewer").await;
    let b = live_broadcast(&app, &owner).await;
    charge(&db, viewer_id, 1000).await;

    // 지갑 행을 잠가 두 요청이 모두 차감 UPDATE에서 기다리게 한 뒤(=실제로 겹침) 함께 풀어 준다.
    // 잠금·확인은 풀 밖의 별도 커넥션으로 한다: 테스트 풀은 병렬 테스트 전체가 상한 20을 나눠 써서,
    // 잠금을 쥔 채 풀에서 커넥션을 더 기다리면 서로 막힌다(T22 트러블슈팅).
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
            // 트랜잭션 안에서는 pg_stat_activity가 첫 조회 스냅숏으로 고정돼 매번 비운다.
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
        donate(&app, Some(&viewer), b, gift(1000)),
        donate(&app, Some(&viewer), b, gift(1000)),
        release
    );
    let mut results = [code(&r1), code(&r2)];
    results.sort();
    assert_eq!(
        results,
        [
            (StatusCode::CREATED, ""),
            (StatusCode::CONFLICT, "WALLET_INSUFFICIENT_BALANCE")
        ],
        "{} {}",
        r1.json,
        r2.json
    );
    assert_eq!(balance(&db, viewer_id).await, 0);
    assert_eq!(donation_count(&db, viewer_id).await, 1);
    assert_eq!(
        scalar(
            &db,
            "SELECT count(*) FROM ledger_entries WHERE user_id = $1",
            viewer_id
        )
        .await,
        2 // 충전 1 + 후원 1
    );
    assert_consistent(&db).await;
}

#[sqlx::test]
async fn banned_user_cannot_donate(db: PgPool) {
    let (app, _) = setup(&db).await;
    let (owner, owner_id) = login_as(&app, "streamer1").await;
    let (viewer, viewer_id) = login_as(&app, "viewer").await;
    let (other_owner, _) = login_as(&app, "streamer2").await;
    let b = live_broadcast(&app, &owner).await;
    let b2 = live_broadcast(&app, &other_owner).await;
    charge(&db, viewer_id, 5000).await;
    let ban = |until: Option<&'static str>| {
        let db = db.clone();
        async move {
            sqlx::query(
                "INSERT INTO bans (streamer_id, user_id, until)
                 VALUES ($1, $2, now() + $3::interval)
                 ON CONFLICT (streamer_id, user_id) DO UPDATE SET until = EXCLUDED.until",
            )
            .bind(owner_id)
            .bind(viewer_id)
            .bind(until)
            .execute(&db)
            .await
            .unwrap();
        }
    };

    // 영구 밴 → 403, 잔액 그대로. 다른 스트리머에게는 후원 가능
    ban(None).await;
    let res = donate(&app, Some(&viewer), b, gift(1000)).await;
    assert_eq!(code(&res), (StatusCode::FORBIDDEN, "MODERATION_BANNED"));
    assert_eq!(balance(&db, viewer_id).await, 5000);
    let res = donate(&app, Some(&viewer), b2, gift(1000)).await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.json);
    // 임시차단은 후원 가능
    ban(Some("10 minutes")).await;
    let res = donate(&app, Some(&viewer), b, gift(1000)).await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.json);
    assert_eq!(balance(&db, viewer_id).await, 3000);
    assert_consistent(&db).await;
}

#[sqlx::test]
async fn donation_skip_owner_only(db: PgPool) {
    let (app, addr) = setup(&db).await;
    let (owner, _) = login_as(&app, "streamer1").await;
    let (viewer, viewer_id) = login_as(&app, "viewer").await;
    let b = live_broadcast(&app, &owner).await;
    charge(&db, viewer_id, 1000).await;
    let mut ws = connect(addr, b, Some(ORIGIN), None).await.unwrap();
    let id = donate(&app, Some(&viewer), b, gift(1000)).await.json["donation"]["id"]
        .as_i64()
        .unwrap();
    assert_eq!(
        next_donation_event(&mut ws).await["type"],
        "donation.created"
    );

    let skip = |cookie: Option<String>, id: i64| {
        let app = app.clone();
        async move {
            send(
                &app,
                empty_req(
                    "POST",
                    &format!("/api/v1/donations/{id}/skip"),
                    cookie.as_deref(),
                ),
            )
            .await
        }
    };
    assert_eq!(skip(None, id).await.status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        skip(Some(owner.clone()), 999_999).await.status,
        StatusCode::NOT_FOUND
    );
    let res = skip(Some(viewer.clone()), id).await;
    assert_eq!(code(&res), (StatusCode::FORBIDDEN, "BROADCAST_NOT_OWNER"));
    let res = skip(Some(owner.clone()), id).await;
    assert_eq!(res.status, StatusCode::NO_CONTENT, "{}", res.json);
    // 거절된 스킵은 이벤트가 없고, 소유자 스킵만 모든 연결에
    assert_eq!(
        next_donation_event(&mut ws).await,
        json!({ "type": "donation.skipped", "data": { "id": id } })
    );
    assert_consistent(&db).await;
}

#[sqlx::test]
async fn donation_list_totals_and_pages(db: PgPool) {
    let (app, _) = setup(&db).await;
    let (owner, _) = login_as(&app, "streamer1").await;
    let (viewer, viewer_id) = login_as(&app, "viewer").await;
    let b = live_broadcast(&app, &owner).await;
    charge(&db, viewer_id, 100_000).await;
    for amount in [1000, 2000, 3000] {
        let res = donate(&app, Some(&viewer), b, gift(amount)).await;
        assert_eq!(res.status, StatusCode::CREATED, "{}", res.json);
    }
    // 유료 클러치 2건(T24 경로 대신 원장·후원 기록을 같은 함수로 추가): 합계엔 들어가고 목록엔 없다
    for _ in 0..2 {
        let mut tx = db.begin().await.unwrap();
        let p = wallet::debit(
            &mut tx,
            viewer_id,
            100,
            wallet::DebitKind::Clutch { broadcast_id: b },
        )
        .await
        .unwrap();
        wallet::insert_donation(
            &mut tx,
            &wallet::NewDonation {
                broadcast_id: b,
                donor_id: viewer_id,
                kind: wallet::DonationKind::Clutch,
                amount: 100,
                anonymous: false,
                message: None,
                ledger_entry_id: p.ledger_entry_id,
            },
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();
    }

    let mut amounts = vec![];
    let mut cursor = String::new();
    let mut pages = 0;
    loop {
        let res = list(&app, None, b, &format!("?limit=2{cursor}")).await;
        assert_eq!(res.status, StatusCode::OK, "{}", res.json);
        assert_eq!(res.json["total"], 6200);
        assert_eq!(res.json["count"], 3);
        assert_eq!(res.json["clutchTotal"], 200);
        pages += 1;
        for i in res.json["items"].as_array().unwrap() {
            assert_eq!(i["kind"], "donation");
            amounts.push(i["amount"].as_i64().unwrap());
        }
        match res.json["nextCursor"].as_str() {
            Some(c) => cursor = format!("&cursor={c}"),
            None => break,
        }
    }
    assert_eq!((pages, amounts), (2, vec![3000, 2000, 1000]));
    assert_eq!(
        list(&app, None, b, "?cursor=x").await.status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        list(&app, None, 999_999, "").await.status,
        StatusCode::NOT_FOUND
    );

    // 끝난 방송도 조회, 숨긴 방송은 404
    end_live(&app, &owner, b).await;
    assert_eq!(list(&app, None, b, "").await.json["total"], 6200);
    let res = send(&app, empty_req("DELETE", &format!("{B}/{b}"), Some(&owner))).await;
    assert_eq!(res.status, StatusCode::NO_CONTENT, "{}", res.json);
    assert_eq!(list(&app, None, b, "").await.status, StatusCode::NOT_FOUND);
    assert_consistent(&db).await;
}

#[sqlx::test]
async fn donation_stats_kst_7_days(db: PgPool) {
    let (app, _) = setup(&db).await;
    let (owner, owner_id) = login_as(&app, "streamer1").await;
    let (viewer, viewer_id) = login_as(&app, "viewer").await;
    let (other_owner, _) = login_as(&app, "streamer2").await;
    charge(&db, viewer_id, 100_000).await;

    // b1(끝남): 7일 경계 직전 1000 + 직후 2000, b2(숨김): 4000, b3(LIVE): 익명 3000, b4(READY): 제외
    let b1 = live_broadcast(&app, &owner).await;
    let d_before = donate(&app, Some(&viewer), b1, gift(1000)).await;
    let d_after = donate(&app, Some(&viewer), b1, gift(2000)).await;
    end_live(&app, &owner, b1).await;
    let b2 = live_broadcast(&app, &owner).await;
    assert_eq!(
        donate(&app, Some(&viewer), b2, gift(4000)).await.status,
        StatusCode::CREATED
    );
    end_live(&app, &owner, b2).await;
    let res = send(
        &app,
        empty_req("DELETE", &format!("{B}/{b2}"), Some(&owner)),
    )
    .await;
    assert_eq!(res.status, StatusCode::NO_CONTENT, "{}", res.json);
    // 받은 기록이 없는 숨긴 방송은 목록에 없다
    let b_empty = live_broadcast(&app, &owner).await;
    end_live(&app, &owner, b_empty).await;
    let res = send(
        &app,
        empty_req("DELETE", &format!("{B}/{b_empty}"), Some(&owner)),
    )
    .await;
    assert_eq!(res.status, StatusCode::NO_CONTENT, "{}", res.json);
    let b3 = live_broadcast(&app, &owner).await;
    let body = json!({ "amount": 3000, "anonymous": true });
    assert_eq!(
        donate(&app, Some(&viewer), b3, body).await.status,
        StatusCode::CREATED
    );
    create(&app, &owner).await;
    // 다른 스트리머가 받은 후원은 섞이지 않는다
    let other_b = live_broadcast(&app, &other_owner).await;
    assert_eq!(
        donate(&app, Some(&viewer), other_b, gift(5000))
            .await
            .status,
        StatusCode::CREATED
    );

    // KST 오늘 포함 7일의 시작(6일 전 00:00 KST) 직전·직후로 옮긴다
    let nine = TimeDelta::hours(9);
    let start = ((Utc::now() + nine).date_naive() - Days::new(6))
        .and_time(NaiveTime::MIN)
        .and_utc()
        - nine;
    for (res, at) in [
        (&d_before, start - TimeDelta::seconds(1)),
        (&d_after, start),
    ] {
        sqlx::query("UPDATE donations SET created_at = $2 WHERE id = $1")
            .bind(res.json["donation"]["id"].as_i64().unwrap())
            .bind(at)
            .execute(&db)
            .await
            .unwrap();
    }

    assert_eq!(
        send(&app, empty_req("GET", "/api/v1/me/donation-stats", None))
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
    let mut items = vec![];
    let mut cursor = String::new();
    loop {
        let res = send(
            &app,
            empty_req(
                "GET",
                &format!("/api/v1/me/donation-stats?limit=2{cursor}"),
                Some(&owner),
            ),
        )
        .await;
        assert_eq!(res.status, StatusCode::OK, "{}", res.json);
        assert_eq!(res.json["totalReceived"], 10_000);
        assert_eq!(res.json["last7Days"], 9000); // 경계 직전 1000 제외
        items.extend(res.json["items"].as_array().unwrap().clone());
        match res.json["nextCursor"].as_str() {
            Some(c) => cursor = format!("&cursor={c}"),
            None => break,
        }
    }
    let summary: Vec<_> = items
        .iter()
        .map(|i| {
            (
                i["broadcast"]["id"].as_i64().unwrap(),
                i["broadcast"]["title"].as_str().unwrap().to_owned(),
                i["broadcast"]["hidden"].as_bool().unwrap(),
                i["total"].as_i64().unwrap(),
                i["count"].as_i64().unwrap(),
            )
        })
        .collect();
    let t = |s: &str| s.to_owned();
    assert_eq!(
        summary,
        [
            (b3, t("후원 방송"), false, 3000, 1),
            (b2, t("삭제된 방송"), true, 4000, 1),
            (b1, t("후원 방송"), false, 3000, 2),
        ]
    );
    assert!(items[0]["broadcast"]["startedAt"].is_string());
    assert_ne!(owner_id, viewer_id);
    assert_consistent(&db).await;
}

#[sqlx::test]
async fn donation_created_on_charge_and_donate(db: PgPool) {
    let (app, addr) = setup(&db).await;
    let (owner_c, _) = login_as(&app, "streamer1").await;
    let (viewer_c, viewer_id) = login_as(&app, "viewer").await;
    let b = live_broadcast(&app, &owner_c).await;
    let mut owner_ws = connect(addr, b, Some(ORIGIN), Some(&owner_c))
        .await
        .unwrap();
    let mut guest_ws = connect(addr, b, Some(ORIGIN), None).await.unwrap();

    // 잔액 0 → 후원 주문(익명 3000) → 승인 확정 = 충전 + 후원 → 알림 이벤트
    let body = json!({ "purpose": "donation", "broadcastId": b,
                       "donation": { "amount": 3000, "anonymous": true } });
    let res = send(
        &app,
        json_req("POST", "/api/v1/payments/orders", Some(&viewer_c), body),
    )
    .await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.json);
    let order_id = res.json["orderId"].as_str().unwrap().to_owned();
    let amount = res.json["amount"].as_i64().unwrap();
    let confirm = || {
        let body =
            json!({ "paymentKey": "pk_live_donation", "orderId": order_id, "amount": amount });
        let (app, c) = (app.clone(), viewer_c.clone());
        async move {
            send(
                &app,
                json_req("POST", "/api/v1/payments/confirm", Some(&c), body),
            )
            .await
        }
    };
    let res = confirm().await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    let id = res.json["donation"]["id"].as_i64().unwrap();

    let ev = next_donation_event(&mut owner_ws).await;
    assert_eq!(ev["type"], "donation.created");
    assert_eq!(ev["data"]["donation"]["id"], id);
    assert_eq!(ev["data"]["donation"]["donor"]["id"], viewer_id);
    assert_eq!(ev["data"]["total"], 3000);
    let ev = next_donation_event(&mut guest_ws).await;
    assert_eq!(ev["data"]["donation"]["id"], id);
    assert!(ev["data"]["donation"]["donor"].is_null(), "{ev}");

    // 같은 승인 재시도(멱등)는 다시 알리지 않는다: 다음 이벤트가 스킵이어야 한다
    assert_eq!(confirm().await.json["donation"]["id"], id);
    let res = send(
        &app,
        empty_req(
            "POST",
            &format!("/api/v1/donations/{id}/skip"),
            Some(&owner_c),
        ),
    )
    .await;
    assert_eq!(res.status, StatusCode::NO_CONTENT);
    assert_eq!(
        next_donation_event(&mut guest_ws).await["type"],
        "donation.skipped"
    );
    // 충전하고 후원하기도 게이지에 들어간다(재시도해도 한 번)
    let gauge_rows =
        "SELECT count(*) FROM clutch_events WHERE broadcast_id = $1 AND source = 'donation'";
    assert_eq!(scalar(&db, gauge_rows, b).await, 1);
    assert_consistent(&db).await;
}

#[sqlx::test]
async fn donation_stats_only_mine(db: PgPool) {
    let (app, _) = setup(&db).await;
    let (owner, _) = login_as(&app, "streamer1").await;
    let (viewer, viewer_id) = login_as(&app, "viewer").await;
    let b = live_broadcast(&app, &owner).await;
    charge(&db, viewer_id, 1000).await;
    assert_eq!(
        donate(&app, Some(&viewer), b, gift(1000)).await.status,
        StatusCode::CREATED
    );
    let fresh = signup(&app, "fresh01", "새사람").await;
    let res = send(
        &app,
        empty_req("GET", "/api/v1/me/donation-stats", Some(&fresh)),
    )
    .await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(
        res.json,
        json!({ "items": [], "nextCursor": null, "totalReceived": 0, "last7Days": 0 })
    );
    assert_consistent(&db).await;
}
