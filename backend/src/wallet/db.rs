use chrono::{DateTime, Utc};
use sqlx::{PgConnection, PgPool};

use super::{NewDonation, Posted};

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

// ===== 결제 주문 =====

pub struct OrderRow {
    pub id: i64,
    pub order_id: String,
    pub payment_key: Option<String>,
    pub user_id: i64,
    pub purpose: String,
    pub amount: i64,
    pub broadcast_id: Option<i64>,
    pub donation_amount: Option<i64>,
    pub donation_anonymous: Option<bool>,
    pub donation_message: Option<String>,
    pub status: String,
    pub expires_at: DateTime<Utc>,
}

pub struct NewOrder<'a> {
    pub order_id: &'a str,
    pub user_id: i64,
    pub amount: i64,
    /// 후원 주문이면 (방송, 후원액, 익명, 메시지)
    pub donation: Option<(i64, i64, bool, Option<&'a str>)>,
}

/// 30분 뒤 만료되는 대기 주문. 만료 시각을 돌려준다.
pub async fn insert_order(db: &PgPool, o: &NewOrder<'_>) -> Result<DateTime<Utc>, sqlx::Error> {
    let (purpose, broadcast_id, d_amount, d_anon, d_msg) = match o.donation {
        Some((b, a, anon, msg)) => ("donation", Some(b), Some(a), Some(anon), msg),
        None => ("charge", None, None, None, None),
    };
    sqlx::query_scalar!(
        r#"INSERT INTO payment_orders (order_id, user_id, purpose, amount, broadcast_id,
                                       donation_amount, donation_anonymous, donation_message, expires_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now() + interval '30 minutes')
           RETURNING expires_at"#,
        o.order_id,
        o.user_id,
        purpose,
        o.amount,
        broadcast_id,
        d_amount,
        d_anon,
        d_msg
    )
    .fetch_one(db)
    .await
}

pub async fn find_order(
    db: impl sqlx::PgExecutor<'_>,
    order_id: &str,
) -> Result<Option<OrderRow>, sqlx::Error> {
    sqlx::query_as!(
        OrderRow,
        "SELECT id, order_id, payment_key, user_id, purpose, amount, broadcast_id, donation_amount,
                donation_anonymous, donation_message, status, expires_at
         FROM payment_orders WHERE order_id = $1",
        order_id
    )
    .fetch_optional(db)
    .await
}

/// 확정 트랜잭션용: 같은 주문의 확정(승인 응답·웹훅·재시도)을 한 줄로 세운다.
pub async fn lock_order(
    conn: &mut PgConnection,
    order_id: &str,
) -> Result<Option<OrderRow>, sqlx::Error> {
    sqlx::query_as!(
        OrderRow,
        "SELECT id, order_id, payment_key, user_id, purpose, amount, broadcast_id, donation_amount,
                donation_anonymous, donation_message, status, expires_at
         FROM payment_orders WHERE order_id = $1 FOR UPDATE",
        order_id
    )
    .fetch_optional(conn)
    .await
}

/// 주문자가 탈퇴했는지(확정 트랜잭션용).
pub async fn user_withdrawn(conn: &mut PgConnection, user_id: i64) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT withdrawn_at IS NOT NULL AS "withdrawn!" FROM users WHERE id = $1"#,
        user_id
    )
    .fetch_one(conn)
    .await
}

pub async fn mark_done(
    conn: &mut PgConnection,
    id: i64,
    payment_key: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE payment_orders SET status = 'done', payment_key = $2, confirmed_at = now() WHERE id = $1",
        id,
        payment_key
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn expire_orders(db: &PgPool) -> Result<u64, sqlx::Error> {
    Ok(sqlx::query!(
        "UPDATE payment_orders SET status = 'expired' WHERE status = 'pending' AND expires_at <= now()"
    )
    .execute(db)
    .await?
    .rows_affected())
}

/// 숨기지 않은 방송의 `(소유자 id, 상태)`.
pub async fn broadcast_info(db: &PgPool, id: i64) -> Result<Option<(i64, String)>, sqlx::Error> {
    let row = sqlx::query!(
        "SELECT owner_id, status FROM broadcasts WHERE id = $1 AND hidden_at IS NULL",
        id
    )
    .fetch_optional(db)
    .await?;
    Ok(row.map(|r| (r.owner_id, r.status)))
}

/// 확정 트랜잭션 안에서 방송 상태. FOR SHARE라 커밋 전에 종료(UPDATE)되지 않는다.
pub async fn broadcast_status_for_share(
    conn: &mut PgConnection,
    id: i64,
) -> Result<Option<(i64, String)>, sqlx::Error> {
    let row = sqlx::query!(
        "SELECT owner_id, status FROM broadcasts WHERE id = $1 FOR SHARE",
        id
    )
    .fetch_optional(conn)
    .await?;
    Ok(row.map(|r| (r.owner_id, r.status)))
}

// ===== 후원 기록 =====

pub struct DonationRow {
    pub id: i64,
    pub broadcast_id: i64,
    pub donor_id: i64,
    pub nickname: String,
    pub kind: String,
    pub amount: i64,
    pub anonymous: bool,
    pub message: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub async fn insert_donation(
    conn: &mut PgConnection,
    d: &NewDonation<'_>,
) -> Result<DonationRow, sqlx::Error> {
    sqlx::query_as!(
        DonationRow,
        r#"WITH d AS (
               INSERT INTO donations (broadcast_id, donor_id, kind, amount, anonymous, message, ledger_entry_id)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING id, broadcast_id, donor_id, kind, amount, anonymous, message, created_at
           )
           SELECT d.id AS "id!", d.broadcast_id AS "broadcast_id!", d.donor_id AS "donor_id!",
                  CASE WHEN u.withdrawn_at IS NULL THEN u.nickname ELSE '탈퇴한 사용자' END AS "nickname!",
                  d.kind AS "kind!", d.amount AS "amount!", d.anonymous AS "anonymous!",
                  d.message, d.created_at AS "created_at!"
           FROM d JOIN users u ON u.id = d.donor_id"#,
        d.broadcast_id,
        d.donor_id,
        d.kind.as_str(),
        d.amount,
        d.anonymous,
        d.message,
        d.ledger_entry_id
    )
    .fetch_one(conn)
    .await
}

/// 후원 주문이 확정 때 남긴 후원 기록. 충전 원장과 후원 원장은 같은 트랜잭션이라 `created_at`(=트랜잭션
/// 시작 시각 `now()`)이 같다 — 주문 → 후원 직접 연결 컬럼이 없어 이것으로 찾는다.
pub async fn order_donation(
    db: &PgPool,
    payment_order_id: i64,
) -> Result<Option<DonationRow>, sqlx::Error> {
    sqlx::query_as!(
        DonationRow,
        r#"SELECT d.id, d.broadcast_id, d.donor_id,
                  CASE WHEN u.withdrawn_at IS NULL THEN u.nickname ELSE '탈퇴한 사용자' END AS "nickname!",
                  d.kind, d.amount, d.anonymous, d.message, d.created_at
           FROM ledger_entries c
           JOIN payment_orders o ON o.id = c.payment_order_id
           JOIN ledger_entries l ON l.user_id = c.user_id AND l.created_at = c.created_at
                                AND l.type = 'donation' AND l.broadcast_id = o.broadcast_id AND l.id > c.id
           JOIN donations d ON d.ledger_entry_id = l.id
           JOIN users u ON u.id = d.donor_id
           WHERE c.payment_order_id = $1
           ORDER BY l.id LIMIT 1"#,
        payment_order_id
    )
    .fetch_optional(db)
    .await
}

/// 방송이 받은 합계. `count`는 donation·burst 건수.
pub struct Totals {
    pub total: i64,
    pub count: i64,
    pub clutch_total: i64,
}

pub async fn donation_totals(db: &PgPool, broadcast_id: i64) -> Result<Totals, sqlx::Error> {
    sqlx::query_as!(
        Totals,
        r#"SELECT COALESCE(SUM(amount), 0)::bigint AS "total!",
                  COUNT(*) FILTER (WHERE kind IN ('donation', 'burst')) AS "count!",
                  COALESCE(SUM(amount) FILTER (WHERE kind = 'clutch'), 0)::bigint AS "clutch_total!"
           FROM donations WHERE broadcast_id = $1"#,
        broadcast_id
    )
    .fetch_one(db)
    .await
}

/// 방송의 donation·burst 최신순(`before`보다 작은 id). 실명 행 — 공개형 변환은 호출자가 한다.
pub async fn list_donations(
    db: &PgPool,
    broadcast_id: i64,
    before: Option<i64>,
    limit: i64,
) -> Result<Vec<DonationRow>, sqlx::Error> {
    sqlx::query_as!(
        DonationRow,
        r#"SELECT d.id, d.broadcast_id, d.donor_id,
                  CASE WHEN u.withdrawn_at IS NULL THEN u.nickname ELSE '탈퇴한 사용자' END AS "nickname!",
                  d.kind, d.amount, d.anonymous, d.message, d.created_at
           FROM donations d JOIN users u ON u.id = d.donor_id
           WHERE d.broadcast_id = $1 AND d.kind IN ('donation', 'burst')
             AND ($2::bigint IS NULL OR d.id < $2)
           ORDER BY d.id DESC LIMIT $3"#,
        broadcast_id,
        before,
        limit
    )
    .fetch_all(db)
    .await
}

/// 후원의 `(방송 id, 방송 소유자 id)`. 숨긴 방송의 후원은 없음.
pub async fn donation_broadcast(db: &PgPool, id: i64) -> Result<Option<(i64, i64)>, sqlx::Error> {
    let row = sqlx::query!(
        "SELECT d.broadcast_id, b.owner_id FROM donations d JOIN broadcasts b ON b.id = d.broadcast_id
         WHERE d.id = $1 AND b.hidden_at IS NULL",
        id
    )
    .fetch_optional(db)
    .await?;
    Ok(row.map(|r| (r.broadcast_id, r.owner_id)))
}

pub struct StatsRow {
    pub id: i64,
    pub title: String,
    pub hidden: bool,
    pub started_at: DateTime<Utc>,
    pub total: i64,
    pub count: i64,
}

/// 내가 송출한 방송별 받은 합계(id 최신순). 숨긴 방송은 받은 기록이 있을 때만("삭제된 방송").
pub async fn stats(
    db: &PgPool,
    owner_id: i64,
    before: Option<i64>,
    limit: i64,
) -> Result<Vec<StatsRow>, sqlx::Error> {
    sqlx::query_as!(
        StatsRow,
        r#"SELECT b.id,
                  CASE WHEN b.hidden_at IS NULL THEN b.title ELSE '삭제된 방송' END AS "title!",
                  b.hidden_at IS NOT NULL AS "hidden!", b.started_at AS "started_at!",
                  COALESCE(SUM(d.amount), 0)::bigint AS "total!",
                  COUNT(d.id) FILTER (WHERE d.kind IN ('donation', 'burst')) AS "count!"
           FROM broadcasts b LEFT JOIN donations d ON d.broadcast_id = b.id
           WHERE b.owner_id = $1 AND b.started_at IS NOT NULL AND ($2::bigint IS NULL OR b.id < $2)
           GROUP BY b.id
           HAVING b.hidden_at IS NULL OR COUNT(d.id) > 0
           ORDER BY b.id DESC LIMIT $3"#,
        owner_id,
        before,
        limit
    )
    .fetch_all(db)
    .await
}

/// 내 방송들이 받은 `(누적, since 이후)` 합계.
pub async fn received(
    db: &PgPool,
    owner_id: i64,
    since: DateTime<Utc>,
) -> Result<(i64, i64), sqlx::Error> {
    let r = sqlx::query!(
        r#"SELECT COALESCE(SUM(d.amount), 0)::bigint AS "total!",
                  COALESCE(SUM(d.amount) FILTER (WHERE d.created_at >= $2), 0)::bigint AS "recent!"
           FROM donations d JOIN broadcasts b ON b.id = d.broadcast_id
           WHERE b.owner_id = $1"#,
        owner_id,
        since
    )
    .fetch_one(db)
    .await?;
    Ok((r.total, r.recent))
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
