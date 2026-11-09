use chrono::{DateTime, Utc};
use sqlx::PgPool;

/// 권한·상태 확인용 최소 정보. 숨긴 방송은 없는 것으로 본다.
pub struct Target {
    pub owner_id: i64,
    pub status: String,
}

pub async fn target(db: &PgPool, id: i64) -> Result<Option<Target>, sqlx::Error> {
    sqlx::query_as!(
        Target,
        "SELECT owner_id, status FROM broadcasts WHERE id = $1 AND hidden_at IS NULL",
        id
    )
    .fetch_optional(db)
    .await
}

pub enum Claim {
    /// LIVE가 아님(ENDED 등)
    NotLive,
    /// 다른 기기가 송출 중이고 전환 요청이 아님
    Active,
    /// 이 신원이 송출자가 됐다. 값은 밀려난 이전 송출자(없거나 같으면 `None`).
    Claimed(Option<String>),
}

/// 송출자 자리 잡기. 행을 잠가 두 기기가 동시에 요청해도 한쪽만 이긴다.
/// 이전 송출자가 없거나, 같은 신원이거나, 끊겨 유예 중(`disconnected_at`, T08b 웹훅이 기록)이거나,
/// `takeover`면 자리를 넘긴다.
pub async fn claim_publisher(
    db: &PgPool,
    id: i64,
    identity: &str,
    takeover: bool,
) -> Result<Claim, sqlx::Error> {
    let mut tx = db.begin().await?;
    let row = sqlx::query!(
        r#"SELECT status, publisher_identity, disconnected_at IS NOT NULL AS "away!"
           FROM broadcasts WHERE id = $1 FOR UPDATE"#,
        id
    )
    .fetch_optional(&mut *tx)
    .await?;
    let Some(row) = row.filter(|r| r.status == "LIVE") else {
        return Ok(Claim::NotLive);
    };
    let previous = row.publisher_identity.filter(|p| p != identity);
    if previous.is_some() && !row.away && !takeover {
        return Ok(Claim::Active);
    }
    sqlx::query!(
        "UPDATE broadcasts SET publisher_identity = $2 WHERE id = $1",
        id,
        identity
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Claim::Claimed(previous))
}

/// LIVE 방송의 송출 상태. LIVE가 아니면 `None`.
pub struct Live {
    pub id: i64,
    pub owner_id: i64,
    pub publisher_identity: Option<String>,
    pub away: bool,
}

pub async fn live(db: &PgPool, id: i64) -> Result<Option<Live>, sqlx::Error> {
    sqlx::query_as!(
        Live,
        r#"SELECT id, owner_id, publisher_identity, disconnected_at IS NOT NULL AS "away!"
           FROM broadcasts WHERE id = $1 AND status = 'LIVE'"#,
        id
    )
    .fetch_optional(db)
    .await
}

/// 모든 LIVE 방송(재시작 동기화).
pub async fn all_live(db: &PgPool) -> Result<Vec<Live>, sqlx::Error> {
    sqlx::query_as!(
        Live,
        r#"SELECT id, owner_id, publisher_identity, disconnected_at IS NOT NULL AS "away!"
           FROM broadcasts WHERE status = 'LIVE'"#
    )
    .fetch_all(db)
    .await
}

/// 유예 시작(`disconnected_at` 기록). `publisher`가 있으면 그 신원이 송출자일 때만.
/// 이미 유예 중이거나 조건이 안 맞으면 `None`, 기록했으면 그 시각.
pub async fn start_grace(
    db: &PgPool,
    id: i64,
    publisher: Option<&str>,
) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"UPDATE broadcasts SET disconnected_at = now()
           WHERE id = $1 AND status = 'LIVE' AND disconnected_at IS NULL
             AND ($2::text IS NULL OR publisher_identity = $2)
           RETURNING disconnected_at AS "since!""#,
        id,
        publisher
    )
    .fetch_optional(db)
    .await
}

/// 송출자 복귀: 그 신원이 송출자이고 유예 중이면 해제. 해제했으면 true.
pub async fn end_grace(db: &PgPool, id: i64, publisher: &str) -> Result<bool, sqlx::Error> {
    let res = sqlx::query!(
        "UPDATE broadcasts SET disconnected_at = NULL
         WHERE id = $1 AND status = 'LIVE' AND publisher_identity = $2
           AND disconnected_at IS NOT NULL",
        id,
        publisher
    )
    .execute(db)
    .await?;
    Ok(res.rows_affected() == 1)
}

/// 최고 시청자 수는 올라갈 때만 바꾼다.
pub async fn raise_peak(db: &PgPool, id: i64, count: i64) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE broadcasts SET peak_viewers = $2
         WHERE id = $1 AND status = 'LIVE' AND peak_viewers < $2",
        id,
        i32::try_from(count).unwrap_or(i32::MAX)
    )
    .execute(db)
    .await?;
    Ok(())
}

/// LIVE 방송의 시간 규칙 판정(DB `now()` 기준).
pub struct Deadline {
    pub id: i64,
    pub publisher_identity: Option<String>,
    pub ends_at: DateTime<Utc>,
    /// 유예(`grace`초) 지남
    pub grace_over: bool,
    /// 최대 길이 지남
    pub over_max: bool,
    /// 최대 길이 `warn_before`초 전 지남
    pub warn: bool,
}

pub async fn deadlines(
    db: &PgPool,
    grace: i64,
    max: i64,
    warn_before: i64,
) -> Result<Vec<Deadline>, sqlx::Error> {
    sqlx::query_as!(
        Deadline,
        r#"SELECT id, publisher_identity,
                  started_at + $2::bigint * interval '1 second' AS "ends_at!",
                  COALESCE(disconnected_at <= now() - $1::bigint * interval '1 second', false) AS "grace_over!",
                  started_at <= now() - $2::bigint * interval '1 second' AS "over_max!",
                  started_at <= now() - ($2::bigint - $3::bigint) * interval '1 second' AS "warn!"
           FROM broadcasts WHERE status = 'LIVE'"#,
        grace,
        max,
        warn_before
    )
    .fetch_all(db)
    .await
}
