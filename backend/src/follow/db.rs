use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::auth::UserSummary;

/// 탈퇴하지 않은 사용자(없거나 탈퇴면 `None`).
pub async fn active_user(db: &PgPool, id: i64) -> Result<Option<UserSummary>, sqlx::Error> {
    sqlx::query_as!(
        UserSummary,
        "SELECT id, nickname FROM users WHERE id = $1 AND withdrawn_at IS NULL",
        id
    )
    .fetch_optional(db)
    .await
}

pub async fn follower_count(db: &PgPool, streamer_id: i64) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "n!" FROM follows f
           JOIN users u ON u.id = f.follower_id AND u.withdrawn_at IS NULL
           WHERE f.streamer_id = $1"#,
        streamer_id
    )
    .fetch_one(db)
    .await
}

pub async fn is_following(
    db: &PgPool,
    follower_id: i64,
    streamer_id: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM follows WHERE follower_id = $1 AND streamer_id = $2) AS "f!""#,
        follower_id,
        streamer_id
    )
    .fetch_one(db)
    .await
}

pub async fn follow(db: &PgPool, follower_id: i64, streamer_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO follows (follower_id, streamer_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        follower_id,
        streamer_id
    )
    .execute(db)
    .await?;
    Ok(())
}

pub async fn unfollow(db: &PgPool, follower_id: i64, streamer_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM follows WHERE follower_id = $1 AND streamer_id = $2",
        follower_id,
        streamer_id
    )
    .execute(db)
    .await?;
    Ok(())
}

pub struct Followed {
    pub id: i64,
    pub nickname: String,
    pub follower_count: i64,
    pub last_ended_at: Option<DateTime<Utc>>,
}

/// 내가 팔로우한 (탈퇴하지 않은) 스트리머. 마지막 방송이 최근인 순.
pub async fn followed(db: &PgPool, me: i64) -> Result<Vec<Followed>, sqlx::Error> {
    sqlx::query_as!(
        Followed,
        r#"SELECT u.id, u.nickname,
                  (SELECT COUNT(*) FROM follows f2
                   JOIN users fu ON fu.id = f2.follower_id AND fu.withdrawn_at IS NULL
                   WHERE f2.streamer_id = u.id) AS "follower_count!",
                  (SELECT MAX(b.ended_at) FROM broadcasts b
                   WHERE b.owner_id = u.id AND b.status = 'ENDED' AND b.hidden_at IS NULL) AS last_ended_at
           FROM follows f JOIN users u ON u.id = f.streamer_id AND u.withdrawn_at IS NULL
           WHERE f.follower_id = $1
           ORDER BY last_ended_at DESC NULLS LAST, u.id"#,
        me
    )
    .fetch_all(db)
    .await
}
