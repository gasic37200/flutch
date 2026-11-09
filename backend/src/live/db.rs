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
