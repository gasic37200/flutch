//! 지갑·원장: 조건부 차감, 동시 차감, 충전 한도·멱등, 내역 조회(docs/features/06, api.md §6).
//! 모든 시나리오는 끝에서 `mismatched_wallets`(잔액 = 마지막 balance_after = 원장 합계)가 비어야 한다.

use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::http::StatusCode;
use flutch::wallet::{self, DebitKind, Posted, WalletError};
use sqlx::PgPool;

use crate::common::{FakeLiveKit, empty_req, login_as, send, signup, test_state};

fn app(db: &PgPool) -> Router {
    flutch::app(test_state(db.clone(), Arc::new(FakeLiveKit::default())))
}

async fn assert_consistent(db: &PgPool) {
    assert_eq!(
        wallet::mismatched_wallets(db).await.unwrap(),
        Vec::<i64>::new()
    );
}

async fn ledger_count(db: &PgPool, user_id: i64) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM ledger_entries WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(db)
        .await
        .unwrap()
}

async fn balance(db: &PgPool, user_id: i64) -> i64 {
    sqlx::query_scalar("SELECT balance FROM wallets WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(db)
        .await
        .unwrap()
}

async fn order(db: &PgPool, user_id: i64, amount: i64) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO payment_orders (order_id, user_id, purpose, amount, expires_at)
         VALUES (gen_random_uuid()::text, $1, 'charge', $2, now() + interval '30 minutes') RETURNING id",
    )
    .bind(user_id)
    .bind(amount)
    .fetch_one(db)
    .await
    .unwrap()
}

/// 새 주문을 만들어 충전한다.
async fn charge(db: &PgPool, user_id: i64, amount: i64) -> Result<Posted, WalletError> {
    let order_id = order(db, user_id, amount).await;
    let mut tx = db.begin().await.unwrap();
    let r = wallet::credit_charge(&mut tx, user_id, amount, order_id).await;
    tx.commit().await.unwrap();
    r
}

async fn debit(
    db: &PgPool,
    user_id: i64,
    amount: i64,
    kind: DebitKind,
) -> Result<Posted, WalletError> {
    let mut conn = db.acquire().await.unwrap();
    wallet::debit(&mut conn, user_id, amount, kind).await
}

/// ENDED 방송(hidden이면 숨김 처리).
async fn ended_broadcast(db: &PgPool, owner_id: i64, title: &str, hidden: bool) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO broadcasts (owner_id, status, title, category, started_at, ended_at, hidden_at)
         VALUES ($1, 'ENDED', $2, 'fps', now(), now(), CASE WHEN $3 THEN now() END) RETURNING id",
    )
    .bind(owner_id)
    .bind(title)
    .bind(hidden)
    .fetch_one(db)
    .await
    .unwrap()
}

#[sqlx::test]
async fn donation_balance_conditional_debit(db: PgPool) {
    let app = app(&db);
    let (_, viewer) = login_as(&app, "viewer").await;
    let (_, streamer) = login_as(&app, "streamer1").await;
    let b = ended_broadcast(&db, streamer, "방송", false).await;
    let kind = DebitKind::Donation { broadcast_id: b };

    // 잔액 0: 거절, 원장 없음
    assert!(matches!(
        debit(&db, viewer, 1000, kind).await,
        Err(WalletError::Insufficient)
    ));
    assert_eq!(ledger_count(&db, viewer).await, 0);

    charge(&db, viewer, 5000).await.unwrap();
    // 잔액 초과: 거절, 잔액 그대로
    assert!(matches!(
        debit(&db, viewer, 5001, kind).await,
        Err(WalletError::Insufficient)
    ));
    assert_eq!(balance(&db, viewer).await, 5000);
    assert_eq!(ledger_count(&db, viewer).await, 1);

    // 충분: 차감 + 원장 1행, balance_after 일치
    let p = debit(&db, viewer, 3000, kind).await.unwrap();
    assert_eq!(p.balance_after, 2000);
    assert_eq!(balance(&db, viewer).await, 2000);
    let (ty, amount, bid): (String, i64, Option<i64>) =
        sqlx::query_as("SELECT type, amount, broadcast_id FROM ledger_entries WHERE id = $1")
            .bind(p.ledger_entry_id)
            .fetch_one(&db)
            .await
            .unwrap();
    assert_eq!((ty.as_str(), amount, bid), ("donation", -3000, Some(b)));

    // 0·음수 금액은 호출자 버그
    assert!(matches!(
        debit(&db, viewer, 0, kind).await,
        Err(WalletError::InvalidAmount(0))
    ));
    assert!(matches!(
        debit(&db, viewer, -1, kind).await,
        Err(WalletError::InvalidAmount(-1))
    ));
    // 잔액 전부 소멸(탈퇴)
    let p = debit(&db, viewer, 2000, DebitKind::WithdrawForfeit)
        .await
        .unwrap();
    assert_eq!(p.balance_after, 0);
    assert_consistent(&db).await;
}

#[sqlx::test]
async fn wallet_concurrent_debit_only_one_succeeds(db: PgPool) {
    let app = app(&db);
    let (_, viewer) = login_as(&app, "viewer").await;
    let (_, streamer) = login_as(&app, "streamer1").await;
    let b = ended_broadcast(&db, streamer, "방송", false).await;
    charge(&db, viewer, 1000).await.unwrap();

    // 별도 커넥션 두 개의 트랜잭션. 커밋 전에 잠깐 쉬어 두 UPDATE가 반드시 겹치게 한다.
    let run = |kind| {
        let db = db.clone();
        async move {
            let mut tx = db.begin().await.unwrap();
            let r = wallet::debit(&mut tx, viewer, 1000, kind).await;
            tokio::time::sleep(Duration::from_millis(200)).await;
            tx.commit().await.unwrap();
            r
        }
    };
    let (a, c) = tokio::join!(
        run(DebitKind::Donation { broadcast_id: b }),
        run(DebitKind::Clutch { broadcast_id: b })
    );
    let ok = [&a, &c].iter().filter(|r| r.is_ok()).count();
    assert_eq!(ok, 1, "{a:?} {c:?}");
    assert!(
        [&a, &c]
            .iter()
            .any(|r| matches!(r, Err(WalletError::Insufficient)))
    );
    assert_eq!(balance(&db, viewer).await, 0);
    assert_eq!(ledger_count(&db, viewer).await, 2); // 충전 1 + 차감 1
    assert_consistent(&db).await;
}

#[sqlx::test]
async fn wallet_charge_limit_and_idempotent(db: PgPool) {
    let app = app(&db);
    let (_, viewer) = login_as(&app, "viewer").await;

    // 같은 주문 두 번 → 한 번만 적립
    let o = order(&db, viewer, 1000).await;
    let mut conn = db.acquire().await.unwrap();
    wallet::credit_charge(&mut conn, viewer, 1000, o)
        .await
        .unwrap();
    let again = wallet::credit_charge(&mut conn, viewer, 1000, o).await;
    assert!(
        matches!(again, Err(WalletError::AlreadyCredited)),
        "{again:?}"
    );
    assert_eq!(balance(&db, viewer).await, 1000);
    assert_eq!(ledger_count(&db, viewer).await, 1);

    // 상한 3,000,000: 딱 맞으면 성공, 넘으면 거절(잔액·원장 그대로)
    charge(&db, viewer, 1_000_000).await.unwrap();
    charge(&db, viewer, 1_000_000).await.unwrap();
    let p = charge(&db, viewer, 999_000).await.unwrap();
    assert_eq!(p.balance_after, 3_000_000);
    assert!(matches!(
        charge(&db, viewer, 1000).await,
        Err(WalletError::BalanceLimit)
    ));
    assert_eq!(balance(&db, viewer).await, 3_000_000);
    assert_eq!(ledger_count(&db, viewer).await, 4);

    // 상한에서도 이미 적립된 주문은 한도 초과가 아니라 멱등 신호
    let again = wallet::credit_charge(&mut conn, viewer, 1000, o).await;
    assert!(
        matches!(again, Err(WalletError::AlreadyCredited)),
        "{again:?}"
    );
    assert_consistent(&db).await;
}

#[sqlx::test]
async fn wallet_list_filters_and_clutch_grouping(db: PgPool) {
    let app = app(&db);
    let (cookie, viewer) = login_as(&app, "viewer").await;
    let (_, streamer) = login_as(&app, "streamer1").await;
    let b1 = ended_broadcast(&db, streamer, "숨긴 방송", true).await;
    let b2 = ended_broadcast(&db, streamer, "두번째", false).await;
    charge(&db, viewer, 10_000).await.unwrap();
    for _ in 0..3 {
        debit(&db, viewer, 100, DebitKind::Clutch { broadcast_id: b1 })
            .await
            .unwrap();
    }
    debit(&db, viewer, 1000, DebitKind::Donation { broadcast_id: b1 })
        .await
        .unwrap();
    for _ in 0..2 {
        debit(&db, viewer, 100, DebitKind::Clutch { broadcast_id: b1 })
            .await
            .unwrap();
    }
    debit(&db, viewer, 100, DebitKind::Clutch { broadcast_id: b2 })
        .await
        .unwrap();

    // 최신순, limit=2로 페이지 넘기기: [b2 클러치, b1 클러치×2] [후원, b1 클러치×3] [충전]
    let get = |q: String| {
        let (app, cookie) = (app.clone(), cookie.clone());
        async move {
            send(
                &app,
                empty_req("GET", &format!("/api/v1/me/wallet{q}"), Some(&cookie)),
            )
            .await
        }
    };
    let mut pages = vec![];
    let mut cursor: Option<String> = None;
    loop {
        let q = match &cursor {
            Some(c) => format!("?limit=2&cursor={c}"),
            None => "?limit=2".into(),
        };
        let res = get(q).await;
        assert_eq!(res.status, StatusCode::OK, "{}", res.json);
        assert_eq!(res.json["balance"], 10_000 - 1000 - 600);
        pages.push(res.json["items"].as_array().unwrap().clone());
        match res.json["nextCursor"].as_str() {
            Some(c) => cursor = Some(c.to_owned()),
            None => break,
        }
    }
    let items: Vec<_> = pages.concat();
    assert_eq!(pages.len(), 3);
    let summary: Vec<_> = items
        .iter()
        .map(|i| {
            (
                i["type"].as_str().unwrap(),
                i["amount"].as_i64().unwrap(),
                i["count"].as_i64().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        summary,
        [
            ("clutch", -100, 1),
            ("clutch", -200, 2),
            ("donation", -1000, 1),
            ("clutch", -300, 3),
            ("charge", 10_000, 1)
        ]
    );
    assert_eq!(items[0]["balanceAfter"], 8400);
    assert_eq!(items[1]["balanceAfter"], 8500); // 묶음 마지막 원장의 잔액
    assert_eq!(items[0]["broadcast"]["hidden"], false);
    assert_eq!(items[0]["broadcast"]["title"], "두번째");
    assert_eq!(items[1]["broadcast"]["id"], b1);
    assert_eq!(items[1]["broadcast"]["hidden"], true);
    assert_eq!(items[1]["broadcast"]["title"], "삭제된 방송");
    assert!(items[4]["broadcast"].is_null());

    // 유형 필터
    let res = get("?type=donation".into()).await;
    let only: Vec<_> = res.json["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["type"].clone())
        .collect();
    assert_eq!(only, ["donation"]);
    for bad in ["?type=refund", "?month=2026-13", "?cursor=abc", "?limit=x"] {
        assert_eq!(
            get(bad.into()).await.status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "{bad}"
        );
    }
    assert_consistent(&db).await;
}

#[sqlx::test]
async fn wallet_month_filter_kst_boundary(db: PgPool) {
    let app = app(&db);
    let (cookie, viewer) = login_as(&app, "viewer").await;
    // 10/31 14:59:59Z = KST 10/31 23:59:59, 10/31 15:00Z = KST 11/1 00:00 (원장은 추가만이라 시각을 넣어 직접 추가)
    let mut bal = 0;
    for at in ["2026-10-31T14:59:59Z", "2026-10-31T15:00:00Z"] {
        bal += 1000;
        let o = order(&db, viewer, 1000).await;
        sqlx::query(
            "INSERT INTO ledger_entries (user_id, type, amount, balance_after, payment_order_id, created_at)
             VALUES ($1, 'charge', 1000, $2, $3, $4::timestamptz)",
        )
        .bind(viewer)
        .bind(bal)
        .bind(o)
        .bind(at)
        .execute(&db)
        .await
        .unwrap();
    }
    sqlx::query("UPDATE wallets SET balance = $2 WHERE user_id = $1")
        .bind(viewer)
        .bind(bal)
        .execute(&db)
        .await
        .unwrap();

    for (month, want) in [
        ("2026-10", vec![1000]),
        ("2026-11", vec![2000]),
        ("2026-09", vec![]),
    ] {
        let res = send(
            &app,
            empty_req(
                "GET",
                &format!("/api/v1/me/wallet?month={month}"),
                Some(&cookie),
            ),
        )
        .await;
        assert_eq!(res.status, StatusCode::OK);
        let got: Vec<i64> = res.json["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i["balanceAfter"].as_i64().unwrap())
            .collect();
        assert_eq!(got, want, "{month}");
    }
    assert_consistent(&db).await;
}

#[sqlx::test]
async fn wallet_requires_login_and_hides_others(db: PgPool) {
    let app = app(&db);
    let (_, viewer) = login_as(&app, "viewer").await;
    charge(&db, viewer, 5000).await.unwrap();

    let res = send(&app, empty_req("GET", "/api/v1/me/wallet", None)).await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);

    let other = signup(&app, "other01", "다른사람").await;
    let res = send(&app, empty_req("GET", "/api/v1/me/wallet", Some(&other))).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.json["balance"], 0);
    assert_eq!(res.json["items"], serde_json::json!([]));
    assert!(res.json["nextCursor"].is_null());
    assert_consistent(&db).await;
}
