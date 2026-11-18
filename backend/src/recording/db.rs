use chrono::{DateTime, Utc};
use sqlx::PgPool;

/// 방송이 LIVE이고 녹화 켜짐일 때만 행을 만든다(방송당 1개). 이미 있거나 조건이 안 맞으면 `None`.
pub async fn insert_for_live(
    db: &PgPool,
    broadcast_id: i64,
    key: &str,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar!(
        "INSERT INTO recordings (broadcast_id, object_key)
         SELECT id, $2 FROM broadcasts WHERE id = $1 AND status = 'LIVE' AND record
         ON CONFLICT (broadcast_id) DO NOTHING RETURNING id",
        broadcast_id,
        key
    )
    .fetch_optional(db)
    .await
}

pub async fn set_egress(db: &PgPool, id: i64, egress_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE recordings SET egress_id = $2 WHERE id = $1",
        id,
        egress_id
    )
    .execute(db)
    .await
    .map(drop)
}

/// Egress를 시작하지 못했다. 아직 `recording`일 때만.
pub async fn fail_start(db: &PgPool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE recordings SET status = 'failed' WHERE id = $1 AND status = 'recording'",
        id
    )
    .execute(db)
    .await
    .map(drop)
}

pub struct Ref {
    pub id: i64,
    pub status: String,
}

pub async fn ref_of(db: &PgPool, broadcast_id: i64) -> Result<Option<Ref>, sqlx::Error> {
    sqlx::query_as!(
        Ref,
        "SELECT id, status FROM recordings WHERE broadcast_id = $1",
        broadcast_id
    )
    .fetch_optional(db)
    .await
}

/// 진행 중인 Egress id.
pub async fn running_egress(db: &PgPool, broadcast_id: i64) -> Result<Option<String>, sqlx::Error> {
    Ok(sqlx::query_scalar!(
        "SELECT egress_id FROM recordings WHERE broadcast_id = $1 AND status = 'recording'",
        broadcast_id
    )
    .fetch_optional(db)
    .await?
    .flatten())
}

pub struct Finished {
    pub id: i64,
    pub broadcast_id: i64,
}

/// `egress_ended` 성공. 아직 `recording`일 때만 바꾼다(웹훅이 반복돼도 한 번만, 이미 지운 녹화는 되살리지 않는다).
pub async fn finish_ready(
    db: &PgPool,
    egress_id: &str,
    size_bytes: i64,
    duration_sec: i32,
    retention_days: i64,
    video_started_at: Option<DateTime<Utc>>,
) -> Result<Option<Finished>, sqlx::Error> {
    sqlx::query_as!(
        Finished,
        "UPDATE recordings SET status = 'ready', size_bytes = $2, duration_sec = $3,
                expires_at = now() + make_interval(days => $4::int), video_started_at = $5
         WHERE egress_id = $1 AND status = 'recording' RETURNING id, broadcast_id",
        egress_id,
        size_bytes,
        duration_sec,
        retention_days as i32,
        video_started_at,
    )
    .fetch_optional(db)
    .await
}

pub async fn finish_failed(db: &PgPool, egress_id: &str) -> Result<Option<Finished>, sqlx::Error> {
    sqlx::query_as!(
        Finished,
        "UPDATE recordings SET status = 'failed'
         WHERE egress_id = $1 AND status = 'recording' RETURNING id, broadcast_id",
        egress_id
    )
    .fetch_optional(db)
    .await
}

/// 보관 기간이 지난 ready를 deleted로(파일 삭제는 `pending_deletes`가 이어받는다).
pub async fn expire_old(db: &PgPool) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE recordings SET status = 'deleted' WHERE status = 'ready' AND expires_at <= now()"
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

/// 최신 것부터 합계 `max_bytes`까지만 남기고 나머지 ready를 deleted로.
pub async fn expire_over_capacity(db: &PgPool, max_bytes: i64) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE recordings SET status = 'deleted' WHERE id IN (
            SELECT id FROM (
                SELECT id, sum(size_bytes) OVER (ORDER BY created_at DESC, id DESC)::bigint AS running
                FROM recordings WHERE status = 'ready') t
            WHERE running > $1)",
        max_bytes
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

/// 종료 웹훅이 끝내 오지 않은 녹화.
pub async fn fail_stale(db: &PgPool, older_than_secs: f64) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE recordings SET status = 'failed'
         WHERE status = 'recording' AND created_at < now() - make_interval(secs => $1)",
        older_than_secs
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

/// deleted인데 파일이 남은 것(`object_key`가 삭제 대기 표시). 상태를 먼저 바꾸고 파일은 나중에 지운다 —
/// 실패해도 깨진 링크는 생기지 않고, 고아 파일은 다음 주기에 다시 시도해 정리된다.
pub async fn pending_deletes(db: &PgPool) -> Result<Vec<(i64, String)>, sqlx::Error> {
    Ok(sqlx::query!(
        r#"SELECT id, object_key AS "object_key!" FROM recordings
           WHERE status = 'deleted' AND object_key IS NOT NULL ORDER BY id LIMIT 200"#
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|r| (r.id, r.object_key))
    .collect())
}

/// 이 Egress의 녹화가 지워졌고 파일 삭제도 끝난 것으로 표시돼 있으면(키 없음) `(id, 방송 id)`.
pub async fn deleted_without_key(
    db: &PgPool,
    egress_id: &str,
) -> Result<Option<(i64, i64)>, sqlx::Error> {
    Ok(sqlx::query!(
        "SELECT id, broadcast_id FROM recordings
          WHERE egress_id = $1 AND status = 'deleted' AND object_key IS NULL",
        egress_id
    )
    .fetch_optional(db)
    .await?
    .map(|r| (r.id, r.broadcast_id)))
}

/// 지운 녹화를 다시 파일 삭제 대기로(`pending_deletes`가 집는다).
pub async fn requeue_delete(db: &PgPool, id: i64, key: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE recordings SET object_key = $2 WHERE id = $1 AND status = 'deleted'",
        id,
        key
    )
    .execute(db)
    .await
    .map(drop)
}

pub async fn clear_key(db: &PgPool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query!("UPDATE recordings SET object_key = NULL WHERE id = $1", id)
        .execute(db)
        .await
        .map(drop)
}

pub struct Row {
    pub id: i64,
    pub broadcast_id: i64,
    pub owner_id: i64,
    pub owner_nickname: String,
    pub title: String,
    pub category: String,
    pub thumbnail_path: Option<String>,
    pub peak_viewers: i32,
    pub views: i64,
    pub started_at: DateTime<Utc>,
    pub status: String,
    pub duration_sec: Option<i32>,
    pub object_key: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// 조회 조건. 숨긴 방송은 항상 뺀다.
#[derive(Default)]
pub struct Filter {
    pub id: Option<i64>,
    pub owner_id: Option<i64>,
    pub category: Option<String>,
    /// `ILIKE`용 패턴(이스케이프 끝난 값)
    pub q: Option<String>,
    pub before: Option<i64>,
    pub ready_only: bool,
    pub limit: i64,
}

pub async fn find(db: &PgPool, f: &Filter) -> Result<Vec<Row>, sqlx::Error> {
    sqlx::query_as!(
        Row,
        r#"SELECT r.id, r.broadcast_id, b.owner_id,
                  CASE WHEN u.withdrawn_at IS NULL THEN u.nickname ELSE '탈퇴한 사용자' END AS "owner_nickname!",
                  b.title, b.category, b.thumbnail_path, b.peak_viewers, r.views,
                  COALESCE(r.video_started_at, b.started_at, b.created_at) AS "started_at!",
                  r.status, r.duration_sec, r.object_key, r.expires_at
           FROM recordings r
           JOIN broadcasts b ON b.id = r.broadcast_id
           JOIN users u ON u.id = b.owner_id
           WHERE b.hidden_at IS NULL
             AND ($1::bigint IS NULL OR r.id = $1)
             AND ($2::bigint IS NULL OR b.owner_id = $2)
             AND ($3::text IS NULL OR b.category = $3)
             AND ($4::text IS NULL OR b.title ILIKE $4 OR u.nickname ILIKE $4
                  OR EXISTS (SELECT 1 FROM unnest(b.tags) t WHERE t ILIKE $4))
             AND ($5::bigint IS NULL OR r.id < $5)
             AND (NOT $6 OR r.status = 'ready')
           ORDER BY r.id DESC
           LIMIT $7"#,
        f.id,
        f.owner_id,
        f.category,
        f.q,
        f.before,
        f.ready_only,
        f.limit,
    )
    .fetch_all(db)
    .await
}

/// 시청자·KST 날짜당 1회만 조회수를 올린다. 올렸으면 true.
pub async fn add_view(db: &PgPool, id: i64, viewer_key: &str) -> Result<bool, sqlx::Error> {
    let mut tx = db.begin().await?;
    let inserted = sqlx::query!(
        "INSERT INTO recording_views (recording_id, viewer_key) VALUES ($1, $2)
         ON CONFLICT DO NOTHING",
        id,
        viewer_key
    )
    .execute(&mut *tx)
    .await?
    .rows_affected()
        == 1;
    if inserted {
        sqlx::query!("UPDATE recordings SET views = views + 1 WHERE id = $1", id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(inserted)
}
