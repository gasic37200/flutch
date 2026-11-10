//! 토스 결제: 주문·승인·웹훅·만료·"충전하고 후원하기"(docs/features/06, api.md §6). 토스는 FakeToss.
//! 모든 시나리오는 끝에서 `mismatched_wallets`(잔액 = 마지막 balance_after = 원장 합계)가 비어야 한다.

use std::sync::Arc;
use std::sync::atomic::Ordering;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use flutch::toss::TossPayment;
use flutch::wallet;
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::common::{
    FakeConfirm, FakeLiveKit, FakeToss, TestResponse, empty_req, json_req, login_as, send, signup,
    test_state_with_toss,
};

const B: &str = "/api/v1/broadcasts";
const ORDERS: &str = "/api/v1/payments/orders";
const CONFIRM: &str = "/api/v1/payments/confirm";

fn setup(db: &PgPool) -> (Router, Arc<FakeToss>) {
    let toss = Arc::new(FakeToss::default());
    let app = flutch::app(test_state_with_toss(
        db.clone(),
        Arc::new(FakeLiveKit::default()),
        toss.clone(),
    ));
    (app, toss)
}

async fn assert_consistent(db: &PgPool) {
    assert_eq!(
        wallet::mismatched_wallets(db).await.unwrap(),
        Vec::<i64>::new()
    );
}

async fn count(db: &PgPool, sql: &'static str, id: i64) -> i64 {
    sqlx::query_scalar(sql)
        .bind(id)
        .fetch_one(db)
        .await
        .unwrap()
}

async fn ledger_count(db: &PgPool, user_id: i64) -> i64 {
    count(
        db,
        "SELECT count(*) FROM ledger_entries WHERE user_id = $1",
        user_id,
    )
    .await
}

async fn donation_count(db: &PgPool, user_id: i64) -> i64 {
    count(
        db,
        "SELECT count(*) FROM donations WHERE donor_id = $1",
        user_id,
    )
    .await
}

async fn order_status(db: &PgPool, order_id: &str) -> String {
    sqlx::query_scalar("SELECT status FROM payment_orders WHERE order_id = $1")
        .bind(order_id)
        .fetch_one(db)
        .await
        .unwrap()
}

async fn live_broadcast(app: &Router, cookie: &str) -> i64 {
    let body = json!({ "title": "후원 테스트", "category": "game" });
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

async fn end_live(app: &Router, cookie: &str, id: i64) {
    let res = send(
        app,
        empty_req("DELETE", &format!("{B}/{id}/live"), Some(cookie)),
    )
    .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
}

async fn order(app: &Router, cookie: &str, body: Value) -> TestResponse {
    send(app, json_req("POST", ORDERS, Some(cookie), body)).await
}

async fn donation_order(app: &Router, cookie: &str, b: i64, amount: i64) -> TestResponse {
    let body = json!({ "purpose": "donation", "broadcastId": b,
                       "donation": { "amount": amount, "anonymous": false } });
    order(app, cookie, body).await
}

/// 대기 주문을 만들고 `(orderId, 결제 금액)`
fn created(res: &TestResponse) -> (String, i64) {
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.json);
    (
        res.json["orderId"].as_str().unwrap().to_owned(),
        res.json["amount"].as_i64().unwrap(),
    )
}

async fn confirm(
    app: &Router,
    cookie: Option<&str>,
    key: &str,
    order_id: &str,
    amount: i64,
) -> TestResponse {
    let body = json!({ "paymentKey": key, "orderId": order_id, "amount": amount });
    send(app, json_req("POST", CONFIRM, cookie, body)).await
}

/// 토스 웹훅(Origin 없음 — 웹훅은 Origin 검사 예외).
async fn webhook_raw(app: &Router, body: String) -> TestResponse {
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/payments/webhook")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();
    send(app, req).await
}

async fn webhook(app: &Router, key: &str, secret: &str) -> TestResponse {
    let body = json!({ "eventType": "PAYMENT_STATUS_CHANGED", "createdAt": "2026-11-10T10:00:00.000000",
                       "data": { "paymentKey": key, "status": "DONE", "secret": secret } });
    webhook_raw(app, body.to_string()).await
}

/// 충전 주문 + 승인. 결과 JSON.
async fn charge(app: &Router, cookie: &str, amount: i64, key: &str) -> Value {
    let (order_id, n) = created(
        &order(
            app,
            cookie,
            json!({ "purpose": "charge", "amount": amount }),
        )
        .await,
    );
    let res = confirm(app, Some(cookie), key, &order_id, n).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    res.json
}

fn code(res: &TestResponse) -> (StatusCode, &str) {
    (res.status, res.json["error"]["code"].as_str().unwrap_or(""))
}

#[sqlx::test]
async fn donation_confirm_idempotent(db: PgPool) {
    let (app, toss) = setup(&db);
    let (viewer_c, viewer) = login_as(&app, "viewer").await;
    let (streamer_c, _) = login_as(&app, "streamer1").await;
    let b = live_broadcast(&app, &streamer_c).await;
    charge(&app, &viewer_c, 4500, "pk_charge").await;

    // 잔액 4,500에 5,000 후원 → 결제 금액은 서버 계산 max(500, 1,000) = 1,000(클라이언트 amount 무시)
    let body = json!({ "purpose": "donation", "broadcastId": b, "amount": 1,
                       "donation": { "amount": 5000, "message": "  응원해요  ", "anonymous": true } });
    let res = order(&app, &viewer_c, body).await;
    let (order_id, n) = created(&res);
    assert_eq!(n, 1000);
    assert_eq!(res.json["orderName"], "1,000비트 충전 후 후원");
    assert_eq!(res.json["clientKey"], "test_gck_fake");
    assert!(order_id.len() >= 6 && order_id.bytes().all(|c| c.is_ascii_alphanumeric()));

    let first = confirm(&app, Some(&viewer_c), "pk_don", &order_id, n).await;
    assert_eq!(first.status, StatusCode::OK, "{}", first.json);
    let r = &first.json;
    assert_eq!(
        (&r["purpose"], &r["charged"], &r["balance"]),
        (&json!("donation"), &json!(1000), &json!(500))
    );
    assert!(r["notice"].is_null());
    let d = &r["donation"];
    assert_eq!(
        (&d["amount"], &d["anonymous"], &d["message"], &d["kind"]),
        (
            &json!(5000),
            &json!(true),
            &json!("응원해요"),
            &json!("donation")
        )
    );
    assert_eq!(d["donor"]["id"], viewer); // 내 후원이라 실명
    assert_eq!(d["broadcastId"], b);

    // 같은 confirm·웹훅 반복 → 재적립 없음, 같은 결과
    for _ in 0..2 {
        let again = confirm(&app, Some(&viewer_c), "pk_don", &order_id, n).await;
        assert_eq!((again.status, &again.json), (StatusCode::OK, &first.json));
    }
    for _ in 0..3 {
        let res = webhook(&app, "pk_don", &FakeToss::secret("pk_don")).await;
        assert_eq!(res.status, StatusCode::OK);
    }
    let again = confirm(&app, Some(&viewer_c), "pk_don", &order_id, n).await;
    assert_eq!(again.json, first.json);
    // 다른 paymentKey로는 거절
    let other = confirm(&app, Some(&viewer_c), "pk_other", &order_id, n).await;
    assert_eq!(
        code(&other),
        (StatusCode::CONFLICT, "PAYMENT_ALREADY_CONFIRMED")
    );

    assert_eq!(ledger_count(&db, viewer).await, 3); // 충전 4,500 · 충전 1,000 · 후원 -5,000
    assert_eq!(donation_count(&db, viewer).await, 1);
    assert_eq!(toss.confirm_calls.lock().unwrap().len(), 2); // 확정된 주문은 토스를 다시 부르지 않는다
    assert_eq!(order_status(&db, &order_id).await, "done");
    assert_consistent(&db).await;
}

#[sqlx::test]
async fn donation_amount_mismatch_rejected(db: PgPool) {
    let (app, toss) = setup(&db);
    let (viewer_c, viewer) = login_as(&app, "viewer").await;
    let (streamer_c, _) = login_as(&app, "streamer1").await;
    let b = live_broadcast(&app, &streamer_c).await;
    let (order_id, n) = created(&donation_order(&app, &viewer_c, b, 3000).await);
    assert_eq!(n, 3000);

    // 주문과 다른 금액 → 토스 호출 없이 거절
    let res = confirm(&app, Some(&viewer_c), "pk_1", &order_id, 2000).await;
    assert_eq!(
        code(&res),
        (StatusCode::CONFLICT, "PAYMENT_AMOUNT_MISMATCH")
    );
    assert!(toss.confirm_calls.lock().unwrap().is_empty());

    // 토스 쪽 결제 금액이 주문과 다르면 웹훅도 적립하지 않는다
    toss.payments.lock().unwrap().insert(
        "pk_1".into(),
        TossPayment {
            payment_key: "pk_1".into(),
            order_id: order_id.clone(),
            status: "DONE".into(),
            total_amount: 2000,
            secret: Some(FakeToss::secret("pk_1")),
        },
    );
    let res = webhook(&app, "pk_1", &FakeToss::secret("pk_1")).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(ledger_count(&db, viewer).await, 0);
    assert_eq!(order_status(&db, &order_id).await, "pending");

    // 맞는 금액이면 확정
    let res = confirm(&app, Some(&viewer_c), "pk_2", &order_id, n).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    assert_eq!(res.json["donation"]["amount"], 3000);
    assert_eq!(res.json["balance"], 0);
    assert_consistent(&db).await;
}

#[sqlx::test]
async fn donation_after_end_becomes_charge(db: PgPool) {
    let (app, _) = setup(&db);
    let (viewer_c, viewer) = login_as(&app, "viewer").await;
    let (streamer_c, _) = login_as(&app, "streamer1").await;
    let b = live_broadcast(&app, &streamer_c).await;
    let (order_id, n) = created(&donation_order(&app, &viewer_c, b, 2000).await);

    end_live(&app, &streamer_c, b).await;
    let res = confirm(&app, Some(&viewer_c), "pk_end", &order_id, n).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    assert!(res.json["donation"].is_null());
    assert_eq!(res.json["notice"], "방송이 종료되어 잔액으로 충전했어요");
    assert_eq!(
        (&res.json["charged"], &res.json["balance"]),
        (&json!(2000), &json!(2000))
    );
    let again = confirm(&app, Some(&viewer_c), "pk_end", &order_id, n).await;
    assert_eq!(again.json, res.json);
    assert_eq!(donation_count(&db, viewer).await, 0);

    // 끝난 방송·없는 방송·내 방송에는 주문 불가
    let res = donation_order(&app, &viewer_c, b, 5000).await;
    assert_eq!(code(&res), (StatusCode::CONFLICT, "BROADCAST_NOT_LIVE"));
    let res = donation_order(&app, &viewer_c, 999_999, 5000).await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
    let b2 = live_broadcast(&app, &streamer_c).await;
    let res = donation_order(&app, &streamer_c, b2, 5000).await;
    assert_eq!(code(&res), (StatusCode::FORBIDDEN, "DONATION_SELF"));
    assert_consistent(&db).await;
}

#[sqlx::test]
async fn charge_limits_and_order_expiry(db: PgPool) {
    let (app, toss) = setup(&db);
    let (viewer_c, _) = login_as(&app, "viewer").await;
    let (streamer_c, _) = login_as(&app, "streamer1").await;
    let b = live_broadcast(&app, &streamer_c).await;

    for bad in [
        json!({ "purpose": "charge", "amount": 999 }),
        json!({ "purpose": "charge", "amount": 1_000_001 }),
        json!({ "purpose": "refund", "amount": 1000 }),
        json!({ "purpose": "donation", "broadcastId": b, "donation": { "amount": 999, "anonymous": false } }),
        json!({ "purpose": "donation", "broadcastId": b,
                "donation": { "amount": 1000, "message": "가".repeat(101), "anonymous": false } }),
    ] {
        let res = order(&app, &viewer_c, bad.clone()).await;
        assert_eq!(
            res.status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "{bad} {}",
            res.json
        );
    }
    let res = order(
        &app,
        &viewer_c,
        json!({ "purpose": "donation", "broadcastId": b,
        "donation": { "amount": 1000, "message": "토토사이트 홍보", "anonymous": false } }),
    )
    .await;
    assert_eq!(
        code(&res),
        (StatusCode::UNPROCESSABLE_ENTITY, "DONATION_BANNED_WORD")
    );
    let res = send(
        &app,
        json_req(
            "POST",
            ORDERS,
            None,
            json!({ "purpose": "charge", "amount": 1000 }),
        ),
    )
    .await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);

    // 보유 상한 3,000,000: 주문 단계와 승인 직전 모두 확인
    charge(&app, &viewer_c, 1_000_000, "pk_a").await;
    charge(&app, &viewer_c, 1_000_000, "pk_b").await;
    charge(&app, &viewer_c, 998_000, "pk_c").await;
    let res = order(
        &app,
        &viewer_c,
        json!({ "purpose": "charge", "amount": 3000 }),
    )
    .await;
    assert_eq!(code(&res), (StatusCode::CONFLICT, "WALLET_BALANCE_LIMIT"));
    let (late, late_n) = created(
        &order(
            &app,
            &viewer_c,
            json!({ "purpose": "charge", "amount": 2000 }),
        )
        .await,
    );
    charge(&app, &viewer_c, 1000, "pk_d").await; // 2,999,000
    let calls = toss.confirm_calls.lock().unwrap().len();
    let res = confirm(&app, Some(&viewer_c), "pk_late", &late, late_n).await;
    assert_eq!(code(&res), (StatusCode::CONFLICT, "WALLET_BALANCE_LIMIT"));
    assert_eq!(toss.confirm_calls.lock().unwrap().len(), calls); // 토스 승인 전에 거절

    // 잔액이 충분하면 후원 주문 불필요
    let res = donation_order(&app, &viewer_c, b, 5000).await;
    assert_eq!(code(&res), (StatusCode::CONFLICT, "PAYMENT_NOT_NEEDED"));

    // 30분 만료: 승인 요청 거절(토스 호출 없음), 정리 작업이 expired로
    let other = signup(&app, "payer01", "결제자").await;
    let (old, old_n) =
        created(&order(&app, &other, json!({ "purpose": "charge", "amount": 5000 })).await);
    let (fresh, _) =
        created(&order(&app, &other, json!({ "purpose": "charge", "amount": 5000 })).await);
    sqlx::query(
        "UPDATE payment_orders SET expires_at = now() - interval '1 second' WHERE order_id = $1",
    )
    .bind(&old)
    .execute(&db)
    .await
    .unwrap();
    let res = confirm(&app, Some(&other), "pk_old", &old, old_n).await;
    assert_eq!(code(&res), (StatusCode::CONFLICT, "PAYMENT_ORDER_EXPIRED"));
    assert_eq!(wallet::expire_pending_orders(&db).await.unwrap(), 1);
    assert_eq!(order_status(&db, &old).await, "expired");
    assert_eq!(order_status(&db, &fresh).await, "pending");
    let res = confirm(&app, Some(&other), "pk_old", &old, old_n).await;
    assert_eq!(code(&res), (StatusCode::CONFLICT, "PAYMENT_ORDER_EXPIRED"));
    assert!(
        !toss
            .confirm_calls
            .lock()
            .unwrap()
            .contains(&"pk_old".to_owned())
    );
    assert_consistent(&db).await;
}

#[sqlx::test]
async fn banned_user_cannot_donate(db: PgPool) {
    let (app, _) = setup(&db);
    let (viewer_c, viewer) = login_as(&app, "viewer").await;
    let (streamer_c, streamer) = login_as(&app, "streamer1").await;
    let b = live_broadcast(&app, &streamer_c).await;
    let ban = |until: Option<&'static str>| {
        let db = db.clone();
        async move {
            sqlx::query(
                "INSERT INTO bans (streamer_id, user_id, until)
                 VALUES ($1, $2, now() + $3::interval)
                 ON CONFLICT (streamer_id, user_id) DO UPDATE SET until = EXCLUDED.until",
            )
            .bind(streamer)
            .bind(viewer)
            .bind(until)
            .execute(&db)
            .await
            .unwrap();
        }
    };

    // 영구 밴 → 주문 거절
    ban(None).await;
    let res = donation_order(&app, &viewer_c, b, 3000).await;
    assert_eq!(code(&res), (StatusCode::FORBIDDEN, "MODERATION_BANNED"));
    // 임시차단은 후원 가능
    ban(Some("10 minutes")).await;
    let (order_id, n) = created(&donation_order(&app, &viewer_c, b, 3000).await);
    // 주문 뒤 confirm 전에 영구 밴 → 충전만
    ban(None).await;
    let res = confirm(&app, Some(&viewer_c), "pk_ban", &order_id, n).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    assert!(res.json["donation"].is_null());
    assert_eq!(
        res.json["notice"],
        "이 채널에서는 후원할 수 없어 잔액으로 충전했어요"
    );
    assert_eq!(res.json["balance"], 3000);
    let again = confirm(&app, Some(&viewer_c), "pk_ban", &order_id, n).await;
    assert_eq!(again.json, res.json);
    assert_eq!(donation_count(&db, viewer).await, 0);
    assert_consistent(&db).await;
}

#[sqlx::test]
async fn payment_toss_rejected_or_unavailable(db: PgPool) {
    let (app, toss) = setup(&db);
    let (viewer_c, viewer) = login_as(&app, "viewer").await;
    let new_charge = |amount: i64| {
        let (app, c) = (app.clone(), viewer_c.clone());
        async move { created(&order(&app, &c, json!({ "purpose": "charge", "amount": amount })).await) }
    };

    // 거절 → 409, 통신 실패 → 502. 주문은 pending으로 남아 같은 주문으로 다시 시도할 수 있다.
    let (a, an) = new_charge(1000).await;
    *toss.next_confirm.lock().unwrap() = Some(FakeConfirm::Reject);
    let res = confirm(&app, Some(&viewer_c), "pk_a1", &a, an).await;
    assert_eq!(
        code(&res),
        (StatusCode::CONFLICT, "PAYMENT_APPROVAL_FAILED")
    );
    *toss.next_confirm.lock().unwrap() = Some(FakeConfirm::Unavailable);
    let res = confirm(&app, Some(&viewer_c), "pk_a2", &a, an).await;
    assert_eq!(
        code(&res),
        (StatusCode::BAD_GATEWAY, "PAYMENT_PROVIDER_UNAVAILABLE")
    );
    assert_eq!(order_status(&db, &a).await, "pending");
    assert_eq!(ledger_count(&db, viewer).await, 0);
    let res = confirm(&app, Some(&viewer_c), "pk_a3", &a, an).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);

    // 토스는 승인했는데 응답을 못 받음 → 결제 조회로 확인해 확정
    let (b, bn) = new_charge(2000).await;
    *toss.next_confirm.lock().unwrap() = Some(FakeConfirm::ApproveThenUnavailable);
    let res = confirm(&app, Some(&viewer_c), "pk_b", &b, bn).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    assert_eq!(res.json["balance"], 3000);

    // 조회까지 실패하면 502 → 웹훅(조회 재확인)이 복구, 이후 confirm 재시도는 같은 결과
    let (c, cn) = new_charge(4000).await;
    *toss.next_confirm.lock().unwrap() = Some(FakeConfirm::ApproveThenUnavailable);
    toss.lookup_fails.store(true, Ordering::SeqCst);
    let res = confirm(&app, Some(&viewer_c), "pk_c", &c, cn).await;
    assert_eq!(
        code(&res),
        (StatusCode::BAD_GATEWAY, "PAYMENT_PROVIDER_UNAVAILABLE")
    );
    let res = webhook(&app, "pk_c", &FakeToss::secret("pk_c")).await;
    assert_eq!(res.status, StatusCode::INTERNAL_SERVER_ERROR); // 토스가 다시 보낸다
    assert_eq!(order_status(&db, &c).await, "pending");
    toss.lookup_fails.store(false, Ordering::SeqCst);
    let res = webhook(&app, "pk_c", &FakeToss::secret("pk_c")).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(order_status(&db, &c).await, "done");
    let res = confirm(&app, Some(&viewer_c), "pk_c", &c, cn).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    assert_eq!(
        (&res.json["charged"], &res.json["balance"]),
        (&json!(4000), &json!(7000))
    );
    assert_eq!(ledger_count(&db, viewer).await, 3);
    assert_consistent(&db).await;
}

#[sqlx::test]
async fn payment_confirm_owner_only(db: PgPool) {
    let (app, toss) = setup(&db);
    let (viewer_c, viewer) = login_as(&app, "viewer").await;
    let other = signup(&app, "other01", "다른사람").await;
    let (order_id, n) = created(
        &order(
            &app,
            &viewer_c,
            json!({ "purpose": "charge", "amount": 1000 }),
        )
        .await,
    );

    let res = confirm(&app, None, "pk_x", &order_id, n).await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    let res = confirm(&app, Some(&other), "pk_x", &order_id, n).await;
    assert_eq!(code(&res), (StatusCode::FORBIDDEN, "PAYMENT_NOT_OWNER"));
    let res = confirm(&app, Some(&viewer_c), "pk_x", "no-such-order", n).await;
    assert_eq!(
        code(&res),
        (StatusCode::NOT_FOUND, "PAYMENT_ORDER_NOT_FOUND")
    );
    assert!(toss.confirm_calls.lock().unwrap().is_empty());
    assert_eq!(ledger_count(&db, viewer).await, 0);
    assert_consistent(&db).await;
}

#[sqlx::test]
async fn payment_webhook_secret_mismatch_ignored(db: PgPool) {
    let (app, toss) = setup(&db);
    let (viewer_c, viewer) = login_as(&app, "viewer").await;
    let (order_id, n) = created(
        &order(
            &app,
            &viewer_c,
            json!({ "purpose": "charge", "amount": 3000 }),
        )
        .await,
    );
    let approved = |key: &str, status: &str| TossPayment {
        payment_key: key.into(),
        order_id: order_id.clone(),
        status: status.into(),
        total_amount: n,
        secret: Some(FakeToss::secret(key)),
    };
    toss.payments
        .lock()
        .unwrap()
        .insert("pk_w".into(), approved("pk_w", "DONE"));
    toss.payments
        .lock()
        .unwrap()
        .insert("pk_cancel".into(), approved("pk_cancel", "CANCELED"));

    // 무시하는 웹훅: secret 불일치·없는 결제·DONE 아님·다른 이벤트·깨진 본문 — 모두 200, 적립 없음
    for res in [
        webhook(&app, "pk_w", "wrong-secret").await,
        webhook(&app, "pk_none", "x").await,
        webhook(&app, "pk_cancel", &FakeToss::secret("pk_cancel")).await,
        webhook_raw(
            &app,
            json!({ "eventType": "DEPOSIT_CALLBACK", "data": {} }).to_string(),
        )
        .await,
        webhook_raw(&app, "not json".into()).await,
    ] {
        assert_eq!(res.status, StatusCode::OK);
    }
    assert_eq!(ledger_count(&db, viewer).await, 0);
    assert_eq!(order_status(&db, &order_id).await, "pending");

    // 맞는 secret → 확정(Origin 없이도 통과)
    let res = webhook(&app, "pk_w", &FakeToss::secret("pk_w")).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(order_status(&db, &order_id).await, "done");
    let res = send(&app, empty_req("GET", "/api/v1/me/wallet", Some(&viewer_c))).await;
    assert_eq!(res.json["balance"], 3000);
    assert_consistent(&db).await;
}

#[sqlx::test]
async fn payment_concurrent_confirm_credits_once(db: PgPool) {
    let (app, toss) = setup(&db);
    let (viewer_c, viewer) = login_as(&app, "viewer").await;
    let (streamer_c, _) = login_as(&app, "streamer1").await;
    let b = live_broadcast(&app, &streamer_c).await;
    let (order_id, n) = created(&donation_order(&app, &viewer_c, b, 5000).await);

    // 같은 승인 결과가 동시에 두 번(별도 요청·커넥션) + 웹훅. 승인 지연으로 둘 다 확인 단계를 지나게 한다.
    toss.confirm_delay_ms.store(200, Ordering::SeqCst);
    let (r1, r2) = tokio::join!(
        confirm(&app, Some(&viewer_c), "pk_same", &order_id, n),
        confirm(&app, Some(&viewer_c), "pk_same", &order_id, n),
    );
    assert_eq!(
        (r1.status, r2.status),
        (StatusCode::OK, StatusCode::OK),
        "{} {}",
        r1.json,
        r2.json
    );
    assert_eq!(r1.json, r2.json);
    assert_eq!(toss.confirm_calls.lock().unwrap().len(), 2); // 둘 다 승인까지 갔다(실제로 겹침)
    let secret = FakeToss::secret("pk_same");
    let (w, r3) = tokio::join!(
        webhook(&app, "pk_same", &secret),
        confirm(&app, Some(&viewer_c), "pk_same", &order_id, n),
    );
    assert_eq!((w.status, &r3.json), (StatusCode::OK, &r1.json));

    assert_eq!(ledger_count(&db, viewer).await, 2); // 충전 1 + 후원 1
    assert_eq!(donation_count(&db, viewer).await, 1);
    assert_eq!(r1.json["balance"], 0);
    assert_consistent(&db).await;
}

#[sqlx::test]
async fn payment_webhook_withdrawn_user_not_credited(db: PgPool) {
    let (app, toss) = setup(&db);
    let (viewer_c, viewer) = login_as(&app, "viewer").await;
    let (order_id, n) = created(
        &order(
            &app,
            &viewer_c,
            json!({ "purpose": "charge", "amount": 3000 }),
        )
        .await,
    );
    toss.payments.lock().unwrap().insert(
        "pk_wd".into(),
        TossPayment {
            payment_key: "pk_wd".into(),
            order_id: order_id.clone(),
            status: "DONE".into(),
            total_amount: n,
            secret: Some(FakeToss::secret("pk_wd")),
        },
    );
    sqlx::query("UPDATE users SET withdrawn_at = now() WHERE id = $1")
        .bind(viewer)
        .execute(&db)
        .await
        .unwrap();

    // 탈퇴한 주문자의 웹훅: 200(재시도 불필요), 적립 없음, 주문은 pending 유지
    let res = webhook(&app, "pk_wd", &FakeToss::secret("pk_wd")).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(ledger_count(&db, viewer).await, 0);
    assert_eq!(order_status(&db, &order_id).await, "pending");
    assert_consistent(&db).await;
}
