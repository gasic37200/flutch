use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::chat::db::Row as ChatRow;

pub struct BanRow {
    pub id: i64,
    pub user_id: i64,
    pub nickname: String,
    pub until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// 지금 유효한 차단만, 최신순.
pub async fn list(
    db: &PgPool,
    streamer_id: i64,
    before: Option<i64>,
    limit: i64,
) -> Result<Vec<BanRow>, sqlx::Error> {
    sqlx::query_as!(
        BanRow,
        r#"SELECT b.id, b.user_id,
                  CASE WHEN u.withdrawn_at IS NULL THEN u.nickname ELSE '탈퇴한 사용자' END AS "nickname!",
                  b.until, b.created_at
             FROM bans b JOIN users u ON u.id = b.user_id
            WHERE b.streamer_id = $1 AND (b.until IS NULL OR b.until > now())
              AND ($2::bigint IS NULL OR b.id < $2)
            ORDER BY b.id DESC
            LIMIT $3"#,
        streamer_id,
        before,
        limit
    )
    .fetch_all(db)
    .await
}

/// 대상 사용자의 표시 닉네임. 없으면 `None`.
pub async fn nickname(db: &PgPool, user_id: i64) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT CASE WHEN withdrawn_at IS NULL THEN nickname ELSE '탈퇴한 사용자' END AS "n!"
             FROM users WHERE id = $1"#,
        user_id
    )
    .fetch_optional(db)
    .await
}

/// 차단을 덮어쓰고(새 행이라 목록에서 최신) 이력을 추가한다. 영구 밴이면 그 스트리머의 LIVE 방송에서
/// 대상자 채팅을 모두 삭제 처리한다(같은 트랜잭션). 돌려주는 값: `(생성 시각, LIVE 방송 id)`.
pub async fn upsert(
    db: &PgPool,
    streamer_id: i64,
    user_id: i64,
    until: Option<DateTime<Utc>>,
) -> Result<(DateTime<Utc>, Option<i64>), sqlx::Error> {
    let mut tx = db.begin().await?;
    sqlx::query!(
        "DELETE FROM bans WHERE streamer_id = $1 AND user_id = $2",
        streamer_id,
        user_id
    )
    .execute(&mut *tx)
    .await?;
    let created_at = sqlx::query_scalar!(
        "INSERT INTO bans (streamer_id, user_id, until) VALUES ($1, $2, $3)
         ON CONFLICT (streamer_id, user_id) DO UPDATE SET until = EXCLUDED.until, created_at = now()
         RETURNING created_at",
        streamer_id,
        user_id,
        until
    )
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query!(
        "INSERT INTO ban_events (streamer_id, user_id, action, until) VALUES ($1, $2, $3, $4)",
        streamer_id,
        user_id,
        if until.is_some() { "timeout" } else { "ban" },
        until
    )
    .execute(&mut *tx)
    .await?;
    let live = sqlx::query_scalar!(
        "SELECT id FROM broadcasts WHERE owner_id = $1 AND status = 'LIVE' AND hidden_at IS NULL",
        streamer_id
    )
    .fetch_optional(&mut *tx)
    .await?;
    if let (None, Some(b)) = (until, live) {
        sqlx::query!(
            "UPDATE chats SET deleted_at = now()
              WHERE broadcast_id = $1 AND user_id = $2 AND deleted_at IS NULL",
            b,
            user_id
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok((created_at, live))
}

/// 해제. 차단이 있었을 때만 이력을 남긴다.
pub async fn remove(db: &PgPool, streamer_id: i64, user_id: i64) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;
    let r = sqlx::query!(
        "DELETE FROM bans WHERE streamer_id = $1 AND user_id = $2",
        streamer_id,
        user_id
    )
    .execute(&mut *tx)
    .await?;
    if r.rows_affected() > 0 {
        sqlx::query!(
            "INSERT INTO ban_events (streamer_id, user_id, action) VALUES ($1, $2, 'unban')",
            streamer_id,
            user_id
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await
}

/// 이 시청자가 이 스트리머의 방송들에서 쓴 채팅, 최신순.
pub async fn viewer_chats(
    db: &PgPool,
    streamer_id: i64,
    user_id: i64,
    before: Option<i64>,
    limit: i64,
) -> Result<Vec<ChatRow>, sqlx::Error> {
    sqlx::query_as!(
        ChatRow,
        r#"SELECT c.id, c.broadcast_id, c.streamer_id, c.user_id,
                  CASE WHEN u.withdrawn_at IS NULL THEN u.nickname ELSE '탈퇴한 사용자' END AS "nickname!",
                  c.body, c.edited_at, c.deleted_at, c.created_at,
                  b.status AS broadcast_status,
                  EXISTS (SELECT 1 FROM donations d
                           WHERE d.broadcast_id = c.broadcast_id AND d.donor_id = c.user_id
                             AND d.kind IN ('donation', 'burst')) AS "donor!"
             FROM chats c
             JOIN users u ON u.id = c.user_id
             JOIN broadcasts b ON b.id = c.broadcast_id AND b.hidden_at IS NULL
            WHERE c.streamer_id = $1 AND c.user_id = $2 AND ($3::bigint IS NULL OR c.id < $3)
            ORDER BY c.id DESC
            LIMIT $4"#,
        streamer_id,
        user_id,
        before,
        limit
    )
    .fetch_all(db)
    .await
}
