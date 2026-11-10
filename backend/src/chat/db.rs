use chrono::{DateTime, Utc};
use sqlx::PgPool;

use super::{Badge, Chat};
use crate::auth::{UserSummary, mask_profanity};

/// 채팅 한 줄 + 방송 상태. 숨긴 방송의 채팅은 조회되지 않는다.
pub struct Row {
    pub id: i64,
    pub broadcast_id: i64,
    pub streamer_id: i64,
    pub user_id: i64,
    pub nickname: String,
    pub body: String,
    pub edited_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub broadcast_status: String,
    pub donor: bool,
}

impl Row {
    pub fn into_chat(self) -> Chat {
        let deleted = self.deleted_at.is_some();
        let mut badges = Vec::new();
        if self.user_id == self.streamer_id {
            badges.push(Badge::Streamer);
        }
        if self.donor {
            badges.push(Badge::Donor);
        }
        Chat {
            id: self.id,
            broadcast_id: self.broadcast_id,
            author: UserSummary {
                id: self.user_id,
                nickname: self.nickname,
            },
            // 원문은 DB에만. 응답·이벤트에는 가린 결과만 싣고, 삭제된 채팅은 본문을 내리지 않는다.
            body: (!deleted).then(|| mask_profanity(&self.body)),
            edited: self.edited_at.is_some(),
            deleted,
            badges,
            created_at: self.created_at,
        }
    }
}

/// 숨기지 않은 방송의 `(소유자 id, 상태)`. 없으면 `None`.
pub async fn broadcast_info(db: &PgPool, id: i64) -> Result<Option<(i64, String)>, sqlx::Error> {
    let row = sqlx::query!(
        "SELECT owner_id, status FROM broadcasts WHERE id = $1 AND hidden_at IS NULL",
        id
    )
    .fetch_optional(db)
    .await?;
    Ok(row.map(|r| (r.owner_id, r.status)))
}

/// 최신순으로 `limit`개(`before`보다 오래된 것). 호출자가 뒤집어 오래된→최신으로 낸다.
pub async fn list(
    db: &PgPool,
    broadcast_id: i64,
    before: Option<i64>,
    limit: i64,
) -> Result<Vec<Row>, sqlx::Error> {
    sqlx::query_as!(
        Row,
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
            WHERE c.broadcast_id = $1 AND ($2::bigint IS NULL OR c.id < $2)
            ORDER BY c.id DESC
            LIMIT $3"#,
        broadcast_id,
        before,
        limit
    )
    .fetch_all(db)
    .await
}

pub async fn find(db: &PgPool, id: i64) -> Result<Option<Row>, sqlx::Error> {
    sqlx::query_as!(
        Row,
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
            WHERE c.id = $1"#,
        id
    )
    .fetch_optional(db)
    .await
}

/// 이 방송에서 같은 사용자가 최근 30초 안에 같은 본문을 보낸 횟수(삭제된 것 포함).
pub async fn recent_same_body(
    db: &PgPool,
    broadcast_id: i64,
    user_id: i64,
    body: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM chats
            WHERE broadcast_id = $1 AND user_id = $2 AND body = $3
              AND created_at > now() - interval '30 seconds'"#,
        broadcast_id,
        user_id,
        body
    )
    .fetch_one(db)
    .await
}

/// 방송이 LIVE일 때만 저장한다(확인과 저장 사이에 종료되는 경쟁을 막는다). LIVE가 아니면 `None`.
pub async fn insert_if_live(
    db: &PgPool,
    broadcast_id: i64,
    user_id: i64,
    body: &str,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar!(
        "INSERT INTO chats (broadcast_id, streamer_id, user_id, body)
         SELECT b.id, b.owner_id, $2, $3 FROM broadcasts b
          WHERE b.id = $1 AND b.status = 'LIVE' AND b.hidden_at IS NULL
         RETURNING id",
        broadcast_id,
        user_id,
        body
    )
    .fetch_optional(db)
    .await
}

/// 5분 안·삭제 안 됨·LIVE일 때만 고친다. 아니면 `false`.
pub async fn update_body(db: &PgPool, id: i64, body: &str) -> Result<bool, sqlx::Error> {
    let r = sqlx::query!(
        "UPDATE chats c SET body = $2, edited_at = now()
          WHERE c.id = $1 AND c.deleted_at IS NULL
            AND c.created_at > now() - interval '5 minutes'
            AND EXISTS (SELECT 1 FROM broadcasts b WHERE b.id = c.broadcast_id AND b.status = 'LIVE')",
        id,
        body
    )
    .execute(db)
    .await?;
    Ok(r.rows_affected() == 1)
}

/// 삭제 표시. 이미 삭제됐으면 `false`.
pub async fn mark_deleted(db: &PgPool, id: i64) -> Result<bool, sqlx::Error> {
    let r = sqlx::query!(
        "UPDATE chats SET deleted_at = now() WHERE id = $1 AND deleted_at IS NULL",
        id
    )
    .execute(db)
    .await?;
    Ok(r.rows_affected() == 1)
}

/// 지금 유효한 차단(영구는 `until` None). 풀린 임시차단은 없는 것으로 본다.
pub async fn active_ban(
    db: &PgPool,
    streamer_id: i64,
    user_id: i64,
) -> Result<Option<Option<DateTime<Utc>>>, sqlx::Error> {
    let row = sqlx::query!(
        "SELECT until FROM bans
          WHERE streamer_id = $1 AND user_id = $2 AND (until IS NULL OR until > now())",
        streamer_id,
        user_id
    )
    .fetch_optional(db)
    .await?;
    Ok(row.map(|r| r.until))
}
