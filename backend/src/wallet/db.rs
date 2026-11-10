use chrono::{DateTime, Utc};
use sqlx::{PgConnection, PgPool};

use super::Posted;

/// 0행이면 잔액 부족(또는 지갑 없음). 지갑 갱신과 원장 추가가 한 문장이라 둘 중 하나만 반영되지 않는다.
pub async fn debit(
    conn: &mut PgConnection,
    user_id: i64,
    amount: i64,
    ty: &str,
    broadcast_id: Option<i64>,
) -> Result<Option<Posted>, sqlx::Error> {
    sqlx::query_as!(
        Posted,
        r#"WITH w AS (
               UPDATE wallets SET balance = balance - $2::bigint
               WHERE user_id = $1 AND balance >= $2::bigint
               RETURNING balance
           )
           INSERT INTO ledger_entries (user_id, type, amount, balance_after, broadcast_id)
           SELECT $1, $3::text, -$2::bigint, w.balance, $4::bigint FROM w
           RETURNING id AS ledger_entry_id, balance_after"#,
        user_id,
        amount,
        ty,
        broadcast_id
    )
    .fetch_optional(conn)
    .await
}

/// 0행이면 상한 초과. 같은 주문이 이미 적립돼 있으면 유니크 위반 에러.
pub async fn credit(
    conn: &mut PgConnection,
    user_id: i64,
    amount: i64,
    payment_order_id: i64,
) -> Result<Option<Posted>, sqlx::Error> {
    sqlx::query_as!(
        Posted,
        r#"WITH w AS (
               INSERT INTO wallets (user_id, balance) VALUES ($1, $2::bigint)
               ON CONFLICT (user_id) DO UPDATE SET balance = wallets.balance + EXCLUDED.balance
               WHERE wallets.balance + EXCLUDED.balance <= 3000000
               RETURNING balance
           )
           INSERT INTO ledger_entries (user_id, type, amount, balance_after, payment_order_id)
           SELECT $1, 'charge', $2::bigint, w.balance, $3 FROM w
           RETURNING id AS ledger_entry_id, balance_after"#,
        user_id,
        amount,
        payment_order_id
    )
    .fetch_optional(conn)
    .await
}

pub async fn is_credited(
    conn: &mut PgConnection,
    payment_order_id: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM ledger_entries WHERE payment_order_id = $1) AS "e!""#,
        payment_order_id
    )
    .fetch_one(conn)
    .await
}

/// 합계는 DB numeric으로 비교해 오버플로가 없다.
pub async fn mismatched_wallets(db: &PgPool) -> Result<Vec<i64>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT w.user_id FROM wallets w
           WHERE w.balance <> COALESCE((SELECT l.balance_after FROM ledger_entries l
                                        WHERE l.user_id = w.user_id ORDER BY l.id DESC LIMIT 1), 0)
              OR w.balance <> (SELECT COALESCE(SUM(l.amount), 0) FROM ledger_entries l
                               WHERE l.user_id = w.user_id)
           ORDER BY w.user_id"#
    )
    .fetch_all(db)
    .await
}

pub async fn balance(db: &PgPool, user_id: i64) -> Result<i64, sqlx::Error> {
    Ok(
        sqlx::query_scalar!("SELECT balance FROM wallets WHERE user_id = $1", user_id)
            .fetch_optional(db)
            .await?
            .unwrap_or(0),
    )
}

pub struct EntryRow {
    pub id: i64,
    pub first_id: i64,
    pub ty: String,
    pub amount: i64,
    pub balance_after: i64,
    pub count: i64,
    pub created_at: DateTime<Utc>,
    pub broadcast_id: Option<i64>,
    pub broadcast_title: Option<String>,
    pub broadcast_hidden: Option<bool>,
}

/// 조회 조건. `None`은 조건 없음. `before`는 이전 페이지 마지막 묶음의 첫 원장 id.
pub struct Filter {
    pub user_id: i64,
    pub ty: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub before: Option<i64>,
    pub limit: i64,
}

/// 최신순 내역. 같은 방송의 연속된 clutch 원장은 한 줄로 묶는다(연속 구간마다 그룹 번호).
/// shortcut: 커서 이전의 내 원장 전체에 창 함수를 돈다, 사용자당 원장이 수만 건이 되면 묶음을 미리 저장한다.
pub async fn entries(db: &PgPool, f: &Filter) -> Result<Vec<EntryRow>, sqlx::Error> {
    sqlx::query_as!(
        EntryRow,
        r#"WITH e AS (
               SELECT l.id, l.type, l.amount, l.balance_after, l.broadcast_id, l.created_at,
                      CASE WHEN l.type = 'clutch'
                                AND lag(l.type) OVER w = 'clutch'
                                AND lag(l.broadcast_id) OVER w = l.broadcast_id
                           THEN 0 ELSE 1 END AS starts
               FROM ledger_entries l
               WHERE l.user_id = $1
                 AND ($2::text IS NULL OR l.type = $2)
                 AND ($3::timestamptz IS NULL OR l.created_at >= $3)
                 AND ($4::timestamptz IS NULL OR l.created_at < $4)
                 AND ($5::bigint IS NULL OR l.id < $5)
               WINDOW w AS (ORDER BY l.id)
           ), g AS (
               SELECT max(id) AS id, min(id) AS first_id, min(type) AS ty,
                      sum(amount)::bigint AS amount,
                      (array_agg(balance_after ORDER BY id DESC))[1] AS balance_after,
                      count(*) AS count, max(created_at) AS created_at,
                      min(broadcast_id) AS broadcast_id
               FROM (SELECT *, sum(starts) OVER (ORDER BY id) AS grp FROM e) s
               GROUP BY grp
               ORDER BY max(id) DESC
               LIMIT $6
           )
           SELECT g.id AS "id!", g.first_id AS "first_id!", g.ty AS "ty!", g.amount AS "amount!",
                  g.balance_after AS "balance_after!", g.count AS "count!",
                  g.created_at AS "created_at!", g.broadcast_id,
                  CASE WHEN b.hidden_at IS NULL THEN b.title ELSE '삭제된 방송' END AS broadcast_title,
                  b.hidden_at IS NOT NULL AS broadcast_hidden
           FROM g LEFT JOIN broadcasts b ON b.id = g.broadcast_id
           ORDER BY g.id DESC"#,
        f.user_id,
        f.ty,
        f.from,
        f.until,
        f.before,
        f.limit
    )
    .fetch_all(db)
    .await
}
