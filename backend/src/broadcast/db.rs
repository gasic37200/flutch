use chrono::{DateTime, Utc};
use sqlx::PgPool;

use super::{Broadcast, BroadcastStatus, parse_enum};
use crate::auth::UserSummary;
use crate::error::AppError;

pub struct Row {
    pub id: i64,
    pub owner_id: i64,
    pub owner_nickname: String,
    pub status: String,
    pub title: String,
    pub category: String,
    pub tags: Vec<String>,
    pub record: bool,
    pub pip_corner: String,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub disconnected_at: Option<DateTime<Utc>>,
    pub peak_viewers: i32,
    pub thumbnail_path: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Row {
    /// `viewer_count`는 메모리 값(`LiveState::viewer_count`). LIVE가 아니면 0으로 낸다.
    pub fn into_broadcast(self, viewer_count: i64) -> Result<Broadcast, AppError> {
        let live = self.status == "LIVE";
        Ok(Broadcast {
            id: self.id,
            owner: UserSummary {
                id: self.owner_id,
                nickname: self.owner_nickname,
            },
            status: parse_enum(&self.status)?,
            title: self.title,
            category: parse_enum(&self.category)?,
            tags: self.tags,
            record: self.record,
            pip_corner: parse_enum(&self.pip_corner)?,
            // shortcut: thumbnail_path를 캐시 무효화용 버전 문자열로 쓴다(T12가 그렇게 채운다).
            thumbnail_url: self
                .thumbnail_path
                .map(|v| format!("/media/thumbs/{}.jpg?v={v}", self.id)),
            viewer_count: if live { viewer_count } else { 0 },
            peak_viewers: self.peak_viewers,
            started_at: self.started_at,
            ended_at: self.ended_at,
            created_at: self.created_at,
            reconnecting: self.disconnected_at.is_some(),
            super_clutch_until: None,
            recording: None,
        })
    }
}

/// 조회 조건. `None`은 "조건 없음". 숨긴 방송은 항상 뺀다.
#[derive(Default)]
pub struct Filter {
    pub id: Option<i64>,
    pub owner_id: Option<i64>,
    pub status: Option<&'static str>,
    pub category: Option<String>,
    /// `ILIKE`용 패턴(`%…%`, 이스케이프 끝난 값)
    pub q: Option<String>,
    /// 이 id보다 오래된 것만
    pub before: Option<i64>,
    pub limit: i64,
}

/// 목록·상세 공용 조회. 정렬은 최신 id 순(LIVE 목록의 시청자 수 순은 `handlers::list_live`가 메모리 값으로 다시 정렬).
pub async fn find(db: &PgPool, f: &Filter) -> Result<Vec<Row>, sqlx::Error> {
    sqlx::query_as!(
        Row,
        r#"SELECT b.id, b.owner_id,
                  CASE WHEN u.withdrawn_at IS NULL THEN u.nickname ELSE '탈퇴한 사용자' END AS "owner_nickname!",
                  b.status, b.title, b.category, b.tags, b.record, b.pip_corner,
                  b.started_at, b.ended_at, b.disconnected_at, b.peak_viewers,
                  b.thumbnail_path, b.created_at
           FROM broadcasts b JOIN users u ON u.id = b.owner_id
           WHERE b.hidden_at IS NULL
             AND ($1::bigint IS NULL OR b.id = $1)
             AND ($2::bigint IS NULL OR b.owner_id = $2)
             AND ($3::text IS NULL OR b.status = $3)
             AND ($4::text IS NULL OR b.category = $4)
             AND ($5::text IS NULL OR b.title ILIKE $5 OR u.nickname ILIKE $5
                  OR EXISTS (SELECT 1 FROM unnest(b.tags) t WHERE t ILIKE $5))
             AND ($6::bigint IS NULL OR b.id < $6)
           ORDER BY b.id DESC
           LIMIT $7"#,
        f.id,
        f.owner_id,
        f.status,
        f.category,
        f.q,
        f.before,
        f.limit,
    )
    .fetch_all(db)
    .await
}

/// 숨기지 않은 방송의 현재 상태(없으면 `None`).
pub async fn status_of(db: &PgPool, id: i64) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT status FROM broadcasts WHERE id = $1 AND hidden_at IS NULL",
        id
    )
    .fetch_optional(db)
    .await
}

/// 등록. 사용자 행을 잠가 READY 개수 확인과 INSERT 사이의 경쟁을 막는다. 한도면 `None`.
pub async fn insert_ready(
    db: &PgPool,
    owner_id: i64,
    title: &str,
    category: &str,
    tags: &[String],
    record: bool,
    limit: i64,
) -> Result<Option<i64>, sqlx::Error> {
    let mut tx = db.begin().await?;
    sqlx::query_scalar!("SELECT id FROM users WHERE id = $1 FOR UPDATE", owner_id)
        .fetch_one(&mut *tx)
        .await?;
    let ready = sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM broadcasts WHERE owner_id = $1 AND status = 'READY'"#,
        owner_id
    )
    .fetch_one(&mut *tx)
    .await?;
    if ready >= limit {
        return Ok(None);
    }
    let id = sqlx::query_scalar!(
        "INSERT INTO broadcasts (owner_id, title, category, tags, record)
         VALUES ($1, $2, $3, $4, $5) RETURNING id",
        owner_id,
        title,
        category,
        tags,
        record,
    )
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Some(id))
}

/// 바꿀 값(`None`은 그대로).
#[derive(Default)]
pub struct Patch {
    pub title: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub record: Option<bool>,
    pub pip_corner: Option<String>,
}

/// 현재 상태가 `expected`일 때만 수정한다. 그 사이 상태가 바뀌었으면 false.
pub async fn update_fields(
    db: &PgPool,
    id: i64,
    expected: BroadcastStatus,
    p: &Patch,
) -> Result<bool, sqlx::Error> {
    let expected = match expected {
        BroadcastStatus::Ready => "READY",
        BroadcastStatus::Live => "LIVE",
        BroadcastStatus::Ended => "ENDED",
    };
    let res = sqlx::query!(
        "UPDATE broadcasts SET title = COALESCE($3, title), category = COALESCE($4, category),
                tags = COALESCE($5, tags), record = COALESCE($6, record),
                pip_corner = COALESCE($7, pip_corner)
         WHERE id = $1 AND status = $2 AND hidden_at IS NULL",
        id,
        expected,
        p.title,
        p.category,
        p.tags.as_deref(),
        p.record,
        p.pip_corner,
    )
    .execute(db)
    .await?;
    Ok(res.rows_affected() == 1)
}

/// `READY → LIVE`. 이미 다른 상태면 false. 소유자의 다른 LIVE가 있으면 유니크 위반 에러.
pub async fn mark_live(db: &PgPool, id: i64) -> Result<bool, sqlx::Error> {
    let res = sqlx::query!(
        "UPDATE broadcasts SET status = 'LIVE', started_at = now()
         WHERE id = $1 AND status = 'READY'",
        id
    )
    .execute(db)
    .await?;
    Ok(res.rows_affected() == 1)
}

pub fn is_live_limit(e: &sqlx::Error) -> bool {
    e.as_database_error().and_then(|d| d.constraint()) == Some("broadcasts_one_live_per_owner")
}

/// `LIVE → ENDED`. LIVE가 아니면 false. `only_if_away`면 유예 중(`disconnected_at`)일 때만 — 방금 돌아온 송출자를 끝내지 않는다.
pub async fn mark_ended(db: &PgPool, id: i64, only_if_away: bool) -> Result<bool, sqlx::Error> {
    let res = sqlx::query!(
        "UPDATE broadcasts SET status = 'ENDED', ended_at = now(), disconnected_at = NULL
         WHERE id = $1 AND status = 'LIVE' AND (NOT $2 OR disconnected_at IS NOT NULL)",
        id,
        only_if_away
    )
    .execute(db)
    .await?;
    Ok(res.rows_affected() == 1)
}

/// READY 행 삭제. READY가 아니면 false.
pub async fn delete_ready(db: &PgPool, id: i64) -> Result<bool, sqlx::Error> {
    let res = sqlx::query!(
        "DELETE FROM broadcasts WHERE id = $1 AND status = 'READY'",
        id
    )
    .execute(db)
    .await?;
    Ok(res.rows_affected() == 1)
}

/// ENDED 방송 숨김 + 녹화를 deleted로. ENDED가 아니거나 이미 숨겼으면 false.
pub async fn hide_ended(db: &PgPool, id: i64) -> Result<bool, sqlx::Error> {
    let mut tx = db.begin().await?;
    let res = sqlx::query!(
        "UPDATE broadcasts SET hidden_at = now()
         WHERE id = $1 AND status = 'ENDED' AND hidden_at IS NULL",
        id
    )
    .execute(&mut *tx)
    .await?;
    if res.rows_affected() == 0 {
        return Ok(false);
    }
    // shortcut: 저장소의 파일 삭제는 T25(녹화)가 붙인다. 여기선 상태만 deleted로.
    sqlx::query!(
        "UPDATE recordings SET status = 'deleted' WHERE broadcast_id = $1",
        id
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(true)
}

/// LIVE인 방송의 썸네일 버전 갱신. LIVE가 아니면 `false`.
pub async fn set_thumbnail(db: &PgPool, id: i64, version: &str) -> Result<bool, sqlx::Error> {
    let res = sqlx::query!(
        "UPDATE broadcasts SET thumbnail_path = $2 WHERE id = $1 AND status = 'LIVE'",
        id,
        version
    )
    .execute(db)
    .await?;
    Ok(res.rows_affected() == 1)
}
