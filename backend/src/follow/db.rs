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
        r#"SELECT COUNT(*) AS "n!" FROM follows WHERE streamer_id = $1"#,
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
