use chrono::{DateTime, Utc};
use sqlx::PgPool;

use super::AuthError;
use crate::error::AppError;
use crate::wallet::{self, DebitKind};

pub struct LoginRow {
    pub id: i64,
    pub password_hash: Option<String>,
    pub locked_until: Option<DateTime<Utc>>,
}

pub struct SessionRow {
    pub user_id: i64,
    pub remember: bool,
    pub expires_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
}

pub struct MeRow {
    pub id: i64,
    pub login_id: Option<String>,
    pub nickname: String,
    pub nickname_changed_at: Option<DateTime<Utc>>,
    pub has_password: bool,
    pub balance: i64,
    pub live_broadcast_id: Option<i64>,
}

pub struct CredentialRow {
    pub login_id: Option<String>,
    pub password_hash: Option<String>,
    pub kakao_id: Option<i64>,
}

/// 사용자와 지갑 행을 함께 만든다(지갑은 가입 시 생성 — 다른 도메인은 행이 있다고 가정).
pub async fn insert_user(
    db: &PgPool,
    login_id: &str,
    password_hash: &str,
    nickname: &str,
) -> Result<i64, AppError> {
    sqlx::query_scalar!(
        r#"WITH u AS (
               INSERT INTO users (login_id, password_hash, nickname) VALUES ($1, $2, $3) RETURNING id
           ), w AS (
               INSERT INTO wallets (user_id) SELECT id FROM u
           )
           SELECT id AS "id!" FROM u"#,
        login_id,
        password_hash,
        nickname
    )
    .fetch_one(db)
    .await
    .map_err(unique_violation)
}

/// 카카오 계정 가입. 아이디 가입과 같이 지갑 행도 함께 만든다.
pub async fn insert_kakao_user(
    db: &PgPool,
    kakao_id: i64,
    nickname: &str,
) -> Result<i64, AppError> {
    sqlx::query_scalar!(
        r#"WITH u AS (
               INSERT INTO users (kakao_id, nickname) VALUES ($1, $2) RETURNING id
           ), w AS (
               INSERT INTO wallets (user_id) SELECT id FROM u
           )
           SELECT id AS "id!" FROM u"#,
        kakao_id,
        nickname
    )
    .fetch_one(db)
    .await
    .map_err(unique_violation)
}

/// 이 카카오 계정의 활성 사용자. 탈퇴하면 `kakao_id`를 비우므로 탈퇴 계정은 나오지 않는다.
pub async fn find_kakao(db: &PgPool, kakao_id: i64) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar!("SELECT id FROM users WHERE kakao_id = $1", kakao_id)
        .fetch_optional(db)
        .await
}

/// 동시 가입·변경이 앞의 중복 확인을 지나쳐도 유니크 제약에서 같은 에러로 바꾼다.
fn unique_violation(e: sqlx::Error) -> AppError {
    match e.as_database_error().and_then(|d| d.constraint()) {
        Some("users_login_id_key") => AuthError::LoginIdTaken.into(),
        Some("users_nickname_active_key") => AuthError::NicknameTaken.into(),
        Some("users_kakao_id_key") => AuthError::KakaoAlreadyRegistered.into(),
        _ => e.into(),
    }
}

/// 탈퇴한 아이디도 영구히 사용 중으로 본다.
pub async fn login_id_exists(db: &PgPool, login_id: &str) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM users WHERE login_id = $1) AS "exists!""#,
        login_id
    )
    .fetch_one(db)
    .await
}

/// 활성 사용자 또는 탈퇴 30일 이내 사용자와 대소문자 무시 중복인지. `except`는 본인.
pub async fn nickname_taken(
    db: &PgPool,
    nickname: &str,
    except: Option<i64>,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT EXISTS (
               SELECT 1 FROM users
               WHERE lower(nickname) = lower($1)
                 AND (withdrawn_at IS NULL OR withdrawn_at > now() - interval '30 days')
                 AND id IS DISTINCT FROM $2
           ) AS "exists!""#,
        nickname,
        except
    )
    .fetch_one(db)
    .await
}

pub async fn find_login(db: &PgPool, login_id: &str) -> Result<Option<LoginRow>, sqlx::Error> {
    sqlx::query_as!(
        LoginRow,
        "SELECT id, password_hash, locked_until FROM users
         WHERE login_id = $1 AND withdrawn_at IS NULL",
        login_id
    )
    .fetch_optional(db)
    .await
}

pub async fn find_active_id(db: &PgPool, login_id: &str) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT id FROM users WHERE login_id = $1 AND withdrawn_at IS NULL",
        login_id
    )
    .fetch_optional(db)
    .await
}

pub async fn existing_login_ids(db: &PgPool, ids: &[String]) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT login_id AS "login_id!" FROM users WHERE login_id = ANY($1)"#,
        ids
    )
    .fetch_all(db)
    .await
}

/// 실패 1회 기록. 5회째면 5분 잠그고 횟수를 0으로 되돌린다. 잠금 시각(있으면)을 돌려준다.
pub async fn record_login_failure(
    db: &PgPool,
    user_id: i64,
) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
    sqlx::query_scalar!(
        "UPDATE users SET
             failed_login_count = CASE WHEN failed_login_count + 1 >= 5 THEN 0 ELSE failed_login_count + 1 END,
             locked_until = CASE WHEN failed_login_count + 1 >= 5 THEN now() + interval '5 minutes' ELSE locked_until END
         WHERE id = $1
         RETURNING locked_until",
        user_id
    )
    .fetch_one(db)
    .await
}

pub async fn reset_login_failures(db: &PgPool, user_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE users SET failed_login_count = 0, locked_until = NULL
         WHERE id = $1 AND (failed_login_count <> 0 OR locked_until IS NOT NULL)",
        user_id
    )
    .execute(db)
    .await?;
    Ok(())
}

pub async fn insert_session(
    db: &PgPool,
    token_hash: &[u8],
    user_id: i64,
    remember: bool,
    expires_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO sessions (token_hash, user_id, remember, expires_at) VALUES ($1, $2, $3, $4)",
        token_hash,
        user_id,
        remember,
        expires_at
    )
    .execute(db)
    .await?;
    Ok(())
}

/// 탈퇴하지 않은 사용자의 유효 세션. 만료됐어도 그 사용자가 LIVE 송출 중이면 유효.
pub async fn find_session(
    db: &PgPool,
    token_hash: &[u8],
) -> Result<Option<SessionRow>, sqlx::Error> {
    sqlx::query_as!(
        SessionRow,
        "SELECT s.user_id, s.remember, s.expires_at, s.last_seen_at
         FROM sessions s JOIN users u ON u.id = s.user_id
         WHERE s.token_hash = $1 AND u.withdrawn_at IS NULL
           AND (s.expires_at > now()
                OR EXISTS (SELECT 1 FROM broadcasts b WHERE b.owner_id = s.user_id AND b.status = 'LIVE'))",
        token_hash
    )
    .fetch_optional(db)
    .await
}

pub async fn extend_session(
    db: &PgPool,
    token_hash: &[u8],
    expires_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE sessions SET last_seen_at = now(), expires_at = $2 WHERE token_hash = $1",
        token_hash,
        expires_at
    )
    .execute(db)
    .await?;
    Ok(())
}

pub async fn delete_session(db: &PgPool, token_hash: &[u8]) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM sessions WHERE token_hash = $1", token_hash)
        .execute(db)
        .await?;
    Ok(())
}

/// 만료 세션 정리. LIVE 송출 중인 사용자의 세션은 남긴다.
pub async fn delete_expired_sessions(db: &PgPool) -> Result<u64, sqlx::Error> {
    let done = sqlx::query!(
        "DELETE FROM sessions s WHERE s.expires_at < now()
           AND NOT EXISTS (SELECT 1 FROM broadcasts b WHERE b.owner_id = s.user_id AND b.status = 'LIVE')"
    )
    .execute(db)
    .await?;
    Ok(done.rows_affected())
}

pub async fn me(db: &PgPool, user_id: i64) -> Result<MeRow, sqlx::Error> {
    sqlx::query_as!(
        MeRow,
        r#"SELECT u.id, u.login_id, u.nickname, u.nickname_changed_at,
                  u.password_hash IS NOT NULL AS "has_password!",
                  COALESCE((SELECT w.balance FROM wallets w WHERE w.user_id = u.id), 0) AS "balance!",
                  (SELECT b.id FROM broadcasts b WHERE b.owner_id = u.id AND b.status = 'LIVE') AS live_broadcast_id
           FROM users u WHERE u.id = $1"#,
        user_id
    )
    .fetch_one(db)
    .await
}

pub async fn credentials(db: &PgPool, user_id: i64) -> Result<CredentialRow, sqlx::Error> {
    sqlx::query_as!(
        CredentialRow,
        "SELECT login_id, password_hash, kakao_id FROM users WHERE id = $1",
        user_id
    )
    .fetch_one(db)
    .await
}

/// 비밀번호를 바꾸고 현재 세션 외 모든 세션을 지운다(한 트랜잭션).
pub async fn change_password(
    db: &PgPool,
    user_id: i64,
    password_hash: &str,
    keep_session: &[u8],
) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;
    sqlx::query!(
        "UPDATE users SET password_hash = $2 WHERE id = $1",
        user_id,
        password_hash
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "DELETE FROM sessions WHERE user_id = $1 AND token_hash <> $2",
        user_id,
        keep_session
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await
}

pub async fn update_nickname(db: &PgPool, user_id: i64, nickname: &str) -> Result<(), AppError> {
    sqlx::query!(
        "UPDATE users SET nickname = $2, nickname_changed_at = now() WHERE id = $1",
        user_id,
        nickname
    )
    .execute(db)
    .await
    .map_err(unique_violation)?;
    Ok(())
}

/// 탈퇴 한 트랜잭션: 잔액 소멸 원장(잔액 > 0일 때만) → `withdrawn_at`(카카오 연결 id 비움) → 세션 전부 삭제. 소멸한 비트 수를 돌려준다.
/// LIVE 방송이 있거나 잔액이 있는데 확인이 없으면 에러(롤백).
pub async fn withdraw(db: &PgPool, user_id: i64, confirm_forfeit: bool) -> Result<i64, AppError> {
    let mut tx = db.begin().await?;
    // 결제 확정(사용자 FOR SHARE)과 직렬화: 확정 중이면 끝난 뒤 그 적립까지 소멸, 탈퇴 중이면 확정이 적립하지 않는다.
    // NO KEY UPDATE: 같은 사용자의 후원·충전이 원장 FK 확인(KEY SHARE)에서 이 잠금을 기다리며 지갑 잠금과 엇갈리지 않게.
    sqlx::query!(
        "SELECT 1 AS one FROM users WHERE id = $1 FOR NO KEY UPDATE",
        user_id
    )
    .fetch_one(&mut *tx)
    .await?;
    // 지갑을 잠가 같은 사용자의 동시 차감·충전이 이 트랜잭션 뒤로 줄 서게 한다.
    let balance = sqlx::query_scalar!(
        "SELECT balance FROM wallets WHERE user_id = $1 FOR UPDATE",
        user_id
    )
    .fetch_optional(&mut *tx)
    .await?
    .unwrap_or(0);
    // READY 방송(채팅·후원 기록이 생길 수 없는 행)은 지운다. 동시에 온 송출 시작은 이 행 잠금을
    // 기다렸다가 행이 없어 실패하고, 먼저 LIVE가 됐으면 아래 확인에서 409가 된다.
    sqlx::query!(
        "DELETE FROM broadcasts WHERE owner_id = $1 AND status = 'READY'",
        user_id
    )
    .execute(&mut *tx)
    .await?;
    let live = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM broadcasts WHERE owner_id = $1 AND status = 'LIVE') AS "e!""#,
        user_id
    )
    .fetch_one(&mut *tx)
    .await?;
    if live {
        return Err(AuthError::WithdrawWhileLive.into());
    }
    if balance > 0 {
        if !confirm_forfeit {
            return Err(AuthError::WithdrawConfirmForfeit(wallet::bits(balance)).into());
        }
        wallet::debit(&mut tx, user_id, balance, DebitKind::WithdrawForfeit).await?;
    }
    sqlx::query!(
        "UPDATE users SET withdrawn_at = now(), kakao_id = NULL WHERE id = $1",
        user_id
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!("DELETE FROM sessions WHERE user_id = $1", user_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(balance)
}
