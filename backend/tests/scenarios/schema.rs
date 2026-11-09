//! 스키마 수준 제약: 앱 코드가 실수해도 DB가 막아야 하는 돈·상태 규칙.

use sqlx::PgPool;

const CHECK_VIOLATION: &str = "23514";
const UNIQUE_VIOLATION: &str = "23505";
const RAISE_EXCEPTION: &str = "P0001";

fn db_code(err: sqlx::Error) -> String {
    err.as_database_error()
        .and_then(|e| e.code())
        .expect("database error")
        .into_owned()
}

async fn user(db: &PgPool, login_id: &str) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO users (login_id, password_hash, nickname) VALUES ($1, 'x', $1) RETURNING id",
    )
    .bind(login_id)
    .fetch_one(db)
    .await
    .unwrap()
}

async fn order(db: &PgPool, user_id: i64, order_id: &str) -> sqlx::Result<i64> {
    sqlx::query_scalar(
        "INSERT INTO payment_orders (order_id, user_id, purpose, amount, expires_at)
         VALUES ($1, $2, 'charge', 1000, now() + interval '30 minutes') RETURNING id",
    )
    .bind(order_id)
    .bind(user_id)
    .fetch_one(db)
    .await
}

#[sqlx::test]
async fn schema_rejects_negative_balance(db: PgPool) {
    let u = user(&db, "viewer1").await;
    sqlx::query("INSERT INTO wallets (user_id, balance) VALUES ($1, 500)")
        .bind(u)
        .execute(&db)
        .await
        .unwrap();

    for bad in [-1_i64, 3_000_001] {
        let err = sqlx::query("UPDATE wallets SET balance = $2 WHERE user_id = $1")
            .bind(u)
            .bind(bad)
            .execute(&db)
            .await
            .unwrap_err();
        assert_eq!(db_code(err), CHECK_VIOLATION, "{bad}");
    }
}

#[sqlx::test]
async fn schema_one_live_per_user(db: PgPool) {
    let u = user(&db, "streamer1").await;
    let insert_live = "INSERT INTO broadcasts (owner_id, status, title, category, started_at)
                       VALUES ($1, 'LIVE', '방송', 'FPS', now())";
    sqlx::query(insert_live).bind(u).execute(&db).await.unwrap();

    let err = sqlx::query(insert_live)
        .bind(u)
        .execute(&db)
        .await
        .unwrap_err();
    assert_eq!(db_code(err), UNIQUE_VIOLATION);

    // 상태와 시각이 어긋난 행(LIVE인데 started_at 없음)도 거절
    let err = sqlx::query(
        "INSERT INTO broadcasts (owner_id, status, title, category) VALUES ($1, 'LIVE', '방송', 'FPS')",
    )
    .bind(user(&db, "streamer2").await)
    .execute(&db)
    .await
    .unwrap_err();
    assert_eq!(db_code(err), CHECK_VIOLATION);
}

#[sqlx::test]
async fn schema_payment_key_unique(db: PgPool) {
    let u = user(&db, "viewer1").await;
    let first = order(&db, u, "order-1").await.unwrap();
    let second = order(&db, u, "order-2").await.unwrap();

    let err = order(&db, u, "order-1").await.unwrap_err();
    assert_eq!(db_code(err), UNIQUE_VIOLATION, "order_id");

    let confirm =
        "UPDATE payment_orders SET status = 'done', payment_key = 'pk-1', confirmed_at = now()
                   WHERE id = $1";
    sqlx::query(confirm).bind(first).execute(&db).await.unwrap();
    let err = sqlx::query(confirm)
        .bind(second)
        .execute(&db)
        .await
        .unwrap_err();
    assert_eq!(db_code(err), UNIQUE_VIOLATION, "payment_key");

    // 같은 주문으로 충전 원장을 두 번 쓸 수 없다
    let charge =
        "INSERT INTO ledger_entries (user_id, type, amount, balance_after, payment_order_id)
                  VALUES ($1, 'charge', 1000, 1000, $2)";
    sqlx::query(charge)
        .bind(u)
        .bind(first)
        .execute(&db)
        .await
        .unwrap();
    let err = sqlx::query(charge)
        .bind(u)
        .bind(first)
        .execute(&db)
        .await
        .unwrap_err();
    assert_eq!(db_code(err), UNIQUE_VIOLATION, "ledger charge per order");
}

#[sqlx::test]
async fn schema_ledger_is_append_only(db: PgPool) {
    let u = user(&db, "viewer1").await;
    let o = order(&db, u, "order-1").await.unwrap();
    sqlx::query(
        "INSERT INTO ledger_entries (user_id, type, amount, balance_after, payment_order_id)
         VALUES ($1, 'charge', 1000, 1000, $2)",
    )
    .bind(u)
    .bind(o)
    .execute(&db)
    .await
    .unwrap();

    for stmt in [
        "UPDATE ledger_entries SET amount = 2000",
        "DELETE FROM ledger_entries",
    ] {
        let err = sqlx::query(stmt).execute(&db).await.unwrap_err();
        assert_eq!(db_code(err), RAISE_EXCEPTION, "{stmt}");
    }

    // 유형과 부호가 맞지 않는 원장(후원인데 +)도 거절
    let b: i64 = sqlx::query_scalar(
        "INSERT INTO broadcasts (owner_id, title, category) VALUES ($1, '방송', 'FPS') RETURNING id",
    )
    .bind(u)
    .fetch_one(&db)
    .await
    .unwrap();
    let err = sqlx::query(
        "INSERT INTO ledger_entries (user_id, type, amount, balance_after, broadcast_id)
         VALUES ($1, 'donation', 1000, 2000, $2)",
    )
    .bind(u)
    .bind(b)
    .execute(&db)
    .await
    .unwrap_err();
    assert_eq!(db_code(err), CHECK_VIOLATION);
}
