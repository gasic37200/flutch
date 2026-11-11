use chrono::{DateTime, Utc};
use sqlx::{PgConnection, PgExecutor, PgPool};

/// 게이지 계산 입력. 시각은 모두 DB 시계(`clock_timestamp()`) 기준이라 테스트의 시간 조작도 같은 기준이다.
pub struct GaugeRow {
    /// 마지막 SUPER CLUTCH 이후 + 최근 창 안의 점수 합
    pub points: i64,
    /// 마지막 SUPER CLUTCH `(발동, 만료)`
    pub last: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub now: DateTime<Utc>,
}

pub async fn gauge_row<'e>(
    ex: impl PgExecutor<'e>,
    broadcast_id: i64,
    window_secs: i64,
) -> Result<GaugeRow, sqlx::Error> {
    let r = sqlx::query!(
        r#"SELECT COALESCE(SUM(e.points), 0)::bigint AS "points!",
                  l.triggered_at AS "triggered_at?", l.expires_at AS "expires_at?",
                  clock_timestamp() AS "now!"
           FROM (SELECT 1) one
           LEFT JOIN LATERAL (
               SELECT triggered_at, expires_at FROM super_clutches
               WHERE broadcast_id = $1 ORDER BY triggered_at DESC LIMIT 1
           ) l ON true
           LEFT JOIN clutch_events e
                  ON e.broadcast_id = $1
                 AND e.created_at > clock_timestamp() - $2::bigint * interval '1 second'
                 AND e.created_at > COALESCE(l.triggered_at, '-infinity')
           GROUP BY l.triggered_at, l.expires_at"#,
        broadcast_id,
        window_secs
    )
    .fetch_one(ex)
    .await?;
    Ok(GaugeRow {
        points: r.points,
        last: r.triggered_at.zip(r.expires_at),
        now: r.now,
    })
}

/// 트랜잭션 끝까지 유지되는 이름 잠금(같은 이름은 직렬화). 같은 세션에서는 다시 잡아도 막히지 않는다.
pub async fn lock(conn: &mut PgConnection, key: &str) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
        .bind(key)
        .execute(conn)
        .await?;
    Ok(())
}

/// 호출 순서가 곧 시간 순서가 되도록 `clock_timestamp()`로 기록한다(트랜잭션 시작 시각 `now()`가 아님).
pub async fn insert_event(
    conn: &mut PgConnection,
    broadcast_id: i64,
    user_id: i64,
    source: &str,
    points: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO clutch_events (broadcast_id, user_id, source, points, created_at)
         VALUES ($1, $2, $3, $4, clock_timestamp())",
        broadcast_id,
        user_id,
        source,
        points
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// 발동 기록. `rotation_order`는 전체 최댓값 + 1(회전 자체는 T29가 이 순서로 돈다).
pub async fn insert_super(
    conn: &mut PgConnection,
    broadcast_id: i64,
    super_secs: i64,
) -> Result<(DateTime<Utc>, DateTime<Utc>), sqlx::Error> {
    let r = sqlx::query!(
        r#"INSERT INTO super_clutches (broadcast_id, triggered_at, expires_at, rotation_order)
           SELECT $1, t, t + $2::bigint * interval '1 second',
                  COALESCE((SELECT MAX(rotation_order) FROM super_clutches), 0) + 1
           FROM (SELECT clock_timestamp() AS t) x
           RETURNING triggered_at, expires_at"#,
        broadcast_id,
        super_secs
    )
    .fetch_one(conn)
    .await?;
    Ok((r.triggered_at, r.expires_at))
}

/// 최근 창 안에서 이 사용자가 쓴 무료 클러치 수.
pub async fn free_used<'e>(
    ex: impl PgExecutor<'e>,
    user_id: i64,
    broadcast_id: i64,
    window_secs: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "n!" FROM clutch_events
           WHERE user_id = $1 AND broadcast_id = $2 AND source = 'free'
             AND created_at > clock_timestamp() - $3::bigint * interval '1 second'"#,
        user_id,
        broadcast_id,
        window_secs
    )
    .fetch_one(ex)
    .await
}

/// 숨기지 않은 방송의 `(소유자 id, 상태)`. FOR SHARE라 커밋 전에 방송이 종료(UPDATE)되지 않는다.
pub async fn broadcast_for_share(
    conn: &mut PgConnection,
    id: i64,
) -> Result<Option<(i64, String)>, sqlx::Error> {
    let r = sqlx::query!(
        "SELECT owner_id, status FROM broadcasts WHERE id = $1 AND hidden_at IS NULL FOR SHARE",
        id
    )
    .fetch_optional(conn)
    .await?;
    Ok(r.map(|r| (r.owner_id, r.status)))
}

/// 진행 중 SUPER CLUTCH 한 건(회전 대상).
pub struct Active {
    pub id: i64,
    pub broadcast_id: i64,
    pub rotation_order: i64,
    /// 마지막 노출 슬롯 시작. `None`이면 아직 노출 전
    pub shown_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub now: DateTime<Utc>,
}

/// 진행 중(미만료 + 방송 LIVE·미숨김)인 발동. 종료된 방송은 여기서 바로 빠진다.
pub async fn active<'e>(ex: impl PgExecutor<'e>) -> Result<Vec<Active>, sqlx::Error> {
    sqlx::query_as!(
        Active,
        r#"SELECT s.id, s.broadcast_id, s.rotation_order, s.shown_at, s.expires_at,
                  clock_timestamp() AS "now!"
           FROM super_clutches s JOIN broadcasts b ON b.id = s.broadcast_id
           WHERE s.expires_at > clock_timestamp() AND b.status = 'LIVE' AND b.hidden_at IS NULL"#
    )
    .fetch_all(ex)
    .await
}

/// 이 발동의 노출 슬롯을 지금 시작한다.
pub async fn mark_shown(conn: &mut PgConnection, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE super_clutches SET shown_at = clock_timestamp() WHERE id = $1",
        id
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub struct ClipRow {
    pub id: i64,
    pub broadcast_id: i64,
    pub recording_id: i64,
    pub object_key: String,
    pub triggered_at: DateTime<Utc>,
    pub start_sec: i32,
    pub duration_sec: i32,
}

/// ready 녹화가 있는 미숨김 방송의 SUPER CLUTCH, 최신순. 구간 = 발동 `before`초 전부터 `len`초(영상 끝을 넘으면 자름).
/// 발동이 녹화 길이를 벗어나 구간이 비면 뺀다.
pub async fn clips(
    db: &PgPool,
    before_id: Option<i64>,
    before: i32,
    len: i32,
    limit: i64,
) -> Result<Vec<ClipRow>, sqlx::Error> {
    sqlx::query_as!(
        ClipRow,
        r#"SELECT s.id, s.broadcast_id, r.id AS recording_id, r.object_key AS "object_key!",
                  s.triggered_at, c.start_sec AS "start_sec!",
                  LEAST($3::int, r.duration_sec - c.start_sec) AS "duration_sec!"
           FROM super_clutches s
           JOIN broadcasts b ON b.id = s.broadcast_id AND b.hidden_at IS NULL
           JOIN recordings r ON r.broadcast_id = b.id
                            AND r.status = 'ready' AND r.object_key IS NOT NULL
           CROSS JOIN LATERAL (
               SELECT GREATEST(0, floor(extract(epoch FROM s.triggered_at - b.started_at))::int - $2::int) AS start_sec
           ) c
           WHERE b.started_at IS NOT NULL
             AND r.duration_sec > c.start_sec
             AND ($1::bigint IS NULL OR s.id < $1)
           ORDER BY s.id DESC
           LIMIT $4"#,
        before_id,
        before,
        len,
        limit
    )
    .fetch_all(db)
    .await
}
