//! 세션 쿠키·토큰과 비밀번호 해시(architecture §7-1 세션 규칙).

use argon2::Argon2;
use argon2::password_hash::phc::PasswordHash;
use argon2::password_hash::{PasswordHasher, PasswordVerifier};
use axum::http::{HeaderMap, HeaderValue, header};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use sqlx::PgPool;

use super::db;
use crate::error::AppError;
use crate::extract::CurrentUser;

pub const COOKIE: &str = "flutch_session";

pub fn lifetime(remember: bool) -> Duration {
    Duration::days(if remember { 30 } else { 7 })
}

/// 32바이트 랜덤 토큰(base64url). 쿠키에는 이 값, DB에는 `hash(token)`만 둔다.
pub fn new_token() -> Result<String, AppError> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).map_err(|e| AppError::Internal(format!("getrandom: {e}")))?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

pub fn hash(token: &str) -> [u8; 32] {
    Sha256::digest(token.as_bytes()).into()
}

/// 요청의 `flutch_session` 쿠키 값.
pub fn token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .find_map(|pair| pair.trim().strip_prefix("flutch_session="))
        .filter(|t| !t.is_empty())
}

/// `Secure`는 로컬 http에서도 붙인다: 브라우저가 `http://localhost`를 보안 출처로 취급해 받아 준다.
pub fn cookie(token: &str, max_age: Duration) -> Result<HeaderValue, AppError> {
    let value = format!(
        "{COOKIE}={token}; Path=/; Max-Age={}; HttpOnly; Secure; SameSite=Lax",
        max_age.num_seconds()
    );
    HeaderValue::from_str(&value).map_err(|e| AppError::Internal(format!("cookie: {e}")))
}

/// 새 세션을 만들고 `Set-Cookie` 값을 돌려준다.
pub async fn start(db: &PgPool, user_id: i64, remember: bool) -> Result<HeaderValue, AppError> {
    let token = new_token()?;
    let ttl = lifetime(remember);
    db::insert_session(db, &hash(&token), user_id, remember, Utc::now() + ttl).await?;
    tracing::Span::current().record("user_id", user_id);
    cookie(&token, ttl)
}

/// 쿠키 토큰 → 로그인 사용자. 만료됐어도 LIVE 송출 중이면 유효. 1시간에 한 번 만료를 연장한다.
pub async fn authenticate(db: &PgPool, token: &str) -> Result<Option<CurrentUser>, AppError> {
    let session_hash = hash(token);
    let Some(s) = db::find_session(db, &session_hash).await? else {
        return Ok(None);
    };
    let now = Utc::now();
    if s.last_seen_at < now - Duration::hours(1) || s.expires_at < now {
        db::extend_session(db, &session_hash, now + lifetime(s.remember)).await?;
    }
    Ok(Some(CurrentUser {
        id: s.user_id,
        remember: s.remember,
        session_hash,
    }))
}

/// argon2 해시는 CPU를 오래 써서 blocking 스레드에서 돈다.
pub async fn hash_password(password: String) -> Result<String, AppError> {
    tokio::task::spawn_blocking(move || {
        Argon2::default()
            .hash_password(password.as_bytes())
            .map(|h| h.to_string())
    })
    .await
    .map_err(|e| AppError::Internal(format!("hash task: {e}")))?
    .map_err(|e| AppError::Internal(format!("argon2 hash: {e}")))
}

pub async fn verify_password(password: String, stored: String) -> Result<bool, AppError> {
    tokio::task::spawn_blocking(move || {
        let parsed = PasswordHash::new(&stored)?;
        Ok::<_, argon2::password_hash::Error>(
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed)
                .is_ok(),
        )
    })
    .await
    .map_err(|e| AppError::Internal(format!("verify task: {e}")))?
    .map_err(|e| AppError::Internal(format!("argon2 parse: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_session_cookie_among_others() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("a=1; flutch_session=tok; b=2"),
        );
        assert_eq!(token(&headers), Some("tok"));
        headers.insert(header::COOKIE, HeaderValue::from_static("flutch_session="));
        assert_eq!(token(&headers), None);
    }

    #[test]
    fn token_is_32_random_bytes() {
        let (a, b) = (new_token().unwrap(), new_token().unwrap());
        assert_eq!(URL_SAFE_NO_PAD.decode(&a).unwrap().len(), 32);
        assert_ne!(a, b);
    }
}
