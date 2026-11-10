//! 카카오 로그인(api.md §2). 입구만 다르고 세션은 아이디 로그인과 같다(decisions §4).
//! 첫 로그인은 카카오 정보를 서명한 임시 쿠키에 담아 닉네임 설정 화면(`/signup/kakao`)으로 보낸다 —
//! 가입이 끝나기 전에는 DB에 아무것도 만들지 않는다.

use std::sync::{Arc, OnceLock};

use axum::extract::{Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{AppendHeaders, IntoResponse, Response};
use axum::{Extension, Json};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::Utc;
use hmac::{Hmac, KeyInit, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use ts_rs::TS;

use super::handlers::{Fields, Limits, check, load_me, require, validated};
use super::rate_limit::client_ip;
use super::{AuthError, KakaoSignupRequest, db, rules, session};
use crate::AppState;
use crate::error::AppError;
use crate::extract::JsonBody;

const AUTHORIZE_URL: &str = "https://kauth.kakao.com/oauth/authorize";
const STATE_COOKIE: &str = "flutch_kakao_state";
const SIGNUP_COOKIE: &str = "flutch_kakao_signup";
/// 두 임시 쿠키는 카카오 라우트에만 보낸다.
const COOKIE_PATH: &str = "/api/v1/auth/kakao";
const TTL_SECS: i64 = 600;

/// 쿠키 값은 모두 base64url·숫자·`.`이라 따로 인코딩하지 않는다. `Lax`라야 카카오에서 돌아오는 이동에 쿠키가 실린다.
fn set_cookie(name: &str, value: &str, max_age: i64) -> Result<HeaderValue, AppError> {
    HeaderValue::from_str(&format!(
        "{name}={value}; Path={COOKIE_PATH}; Max-Age={max_age}; HttpOnly; Secure; SameSite=Lax"
    ))
    .map_err(|e| AppError::Internal(format!("cookie: {e}")))
}

fn read_cookie<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .find_map(|pair| pair.trim().strip_prefix(name)?.strip_prefix('='))
        .filter(|v| !v.is_empty())
}

fn found(to: &str, cookies: impl IntoIterator<Item = HeaderValue>) -> Response {
    (
        StatusCode::FOUND,
        AppendHeaders(cookies.into_iter().map(|c| (header::SET_COOKIE, c))),
        [(header::LOCATION, to)],
    )
        .into_response()
}

/// 임시 쿠키 서명 키. shortcut: 프로세스마다 새로 만든다 — 재시작하면 10분짜리 가입 대기만 무효가 되고,
/// 백엔드를 여러 대로 늘리면 공유 비밀값 환경변수로 바꿔야 한다.
fn key() -> &'static [u8; 32] {
    static KEY: OnceLock<[u8; 32]> = OnceLock::new();
    KEY.get_or_init(|| {
        let mut k = [0u8; 32];
        getrandom::fill(&mut k).expect("getrandom");
        k
    })
}

fn mac(payload: &str) -> Hmac<Sha256> {
    let mut m = Hmac::<Sha256>::new_from_slice(key()).expect("hmac accepts any key length");
    m.update(payload.as_bytes());
    m
}

/// `<카카오 id>.<만료 unix초>.<닉네임 b64>.<HMAC b64>`
fn pack(kakao_id: i64, nickname: &str, exp: i64) -> String {
    let payload = format!("{kakao_id}.{exp}.{}", URL_SAFE_NO_PAD.encode(nickname));
    let sig = URL_SAFE_NO_PAD.encode(mac(&payload).finalize().into_bytes());
    format!("{payload}.{sig}")
}

/// 서명이 맞고 만료 전이면 `(카카오 id, 닉네임 후보)`.
fn unpack(value: &str) -> Option<(i64, String)> {
    let (payload, sig) = value.rsplit_once('.')?;
    mac(payload)
        .verify_slice(&URL_SAFE_NO_PAD.decode(sig).ok()?)
        .ok()?;
    let mut parts = payload.splitn(3, '.');
    let id = parts.next()?.parse().ok()?;
    let exp: i64 = parts.next()?.parse().ok()?;
    if exp <= Utc::now().timestamp() {
        return None;
    }
    let nickname = String::from_utf8(URL_SAFE_NO_PAD.decode(parts.next()?).ok()?).ok()?;
    Some((id, nickname))
}

/// 카카오 닉네임을 우리 규칙(한글·영문·숫자 12자)에 맞는 후보로 줄인다. 최종 검사는 가입 때 다시 한다.
fn candidate(raw: Option<&str>) -> String {
    raw.unwrap_or_default()
        .chars()
        .filter(|c| ('가'..='힣').contains(c) || c.is_ascii_alphanumeric())
        .take(12)
        .collect()
}

fn pending_signup(headers: &HeaderMap) -> Result<(i64, String), AppError> {
    read_cookie(headers, SIGNUP_COOKIE)
        .and_then(unpack)
        .ok_or_else(|| AuthError::KakaoSignupExpired.into())
}

/// 카카오 인가 화면으로 보낸다. 설정이 없으면 404(화면은 이것으로 버튼을 숨긴다).
pub async fn start(State(state): State<Arc<AppState>>) -> Result<Response, AppError> {
    let Some(k) = &state.config.auth.kakao else {
        return Err(AppError::NotFound);
    };
    let st = session::new_token()?;
    let url = reqwest::Url::parse_with_params(
        AUTHORIZE_URL,
        [
            ("client_id", k.rest_key.as_str()),
            ("redirect_uri", k.redirect_uri.as_str()),
            ("response_type", "code"),
            ("scope", "profile_nickname"),
            ("state", st.as_str()),
        ],
    )
    .map_err(|e| AppError::Internal(format!("authorize url: {e}")))?;
    Ok(found(
        url.as_str(),
        [set_cookie(STATE_COOKIE, &st, TTL_SECS)?],
    ))
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    code: Option<String>,
    state: Option<String>,
}

/// 카카오가 돌려보내는 주소. 실패는 모두 로그인 화면(`?kakao=error`)으로 보낸다 — 사용자는 JSON을 보지 않는다.
pub async fn callback(
    State(state): State<Arc<AppState>>,
    Extension(limits): Extension<Arc<Limits>>,
    headers: HeaderMap,
    Query(q): Query<CallbackQuery>,
) -> Result<Response, AppError> {
    let clear_state = set_cookie(STATE_COOKIE, "", 0)?;
    match login(&state, &limits, &headers, q, clear_state.clone()).await {
        Ok(res) => Ok(res),
        Err(e) => {
            tracing::warn!(error = %e, "kakao login failed");
            Ok(found("/login?kakao=error", [clear_state]))
        }
    }
}

async fn login(
    state: &AppState,
    limits: &Limits,
    headers: &HeaderMap,
    q: CallbackQuery,
    clear_state: HeaderValue,
) -> Result<Response, AppError> {
    if state.config.auth.kakao.is_none() {
        return Err(AppError::NotFound);
    }
    limits.login.hit(&client_ip(headers))?;
    // 사용자가 동의를 취소하면 code 없이 error만 온다.
    let (Some(code), Some(returned)) = (q.code, q.state) else {
        return Err(AppError::Forbidden);
    };
    if read_cookie(headers, STATE_COOKIE) != Some(returned.as_str()) {
        return Err(AppError::Forbidden);
    }
    let profile = state
        .kakao
        .profile(&code)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    if let Some(user_id) = db::find_kakao(&state.db, profile.id).await? {
        tracing::info!(user_id, "kakao login");
        let session = session::start(&state.db, user_id, false).await?;
        return Ok(found("/", [session, clear_state]));
    }
    let signup = pack(
        profile.id,
        &candidate(profile.nickname.as_deref()),
        Utc::now().timestamp() + TTL_SECS,
    );
    Ok(found(
        "/signup/kakao",
        [set_cookie(SIGNUP_COOKIE, &signup, TTL_SECS)?, clear_state],
    ))
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct KakaoPending {
    /// 닉네임 설정 화면에 미리 채울 값(비어 있을 수 있다)
    pub nickname: String,
}

/// 닉네임 설정 화면이 미리 채울 값. 임시 쿠키가 없거나 만료·위조면 401.
pub async fn pending(headers: HeaderMap) -> Result<Json<KakaoPending>, AppError> {
    let (_, nickname) = pending_signup(&headers)?;
    Ok(Json(KakaoPending { nickname }))
}

pub async fn signup(
    State(state): State<Arc<AppState>>,
    Extension(limits): Extension<Arc<Limits>>,
    headers: HeaderMap,
    JsonBody(req): JsonBody<KakaoSignupRequest>,
) -> Result<impl IntoResponse, AppError> {
    let (kakao_id, _) = pending_signup(&headers)?;
    let nickname = req.nickname.trim().to_owned();
    let mut fields = Fields::new();
    check(&mut fields, "nickname", rules::check_nickname(&nickname));
    require(
        &mut fields,
        "agreeTerms",
        req.agree_terms,
        "이용약관에 동의해 주세요.",
    );
    require(
        &mut fields,
        "agreePrivacy",
        req.agree_privacy,
        "개인정보처리방침에 동의해 주세요.",
    );
    require(
        &mut fields,
        "over14",
        req.over14,
        "만 14세 이상만 가입할 수 있어요.",
    );
    validated(fields)?;

    limits.signup.hit(&client_ip(&headers))?;
    if db::nickname_taken(&state.db, &nickname, None).await? {
        return Err(AuthError::NicknameTaken.into());
    }
    // 이미 가입된 카카오 계정(다른 탭에서 먼저 끝냄)은 유니크 제약이 409로 바꾼다.
    let user_id = db::insert_kakao_user(&state.db, kakao_id, &nickname).await?;
    tracing::info!(user_id, "user signed up with kakao");
    let session = session::start(&state.db, user_id, false).await?;
    Ok((
        StatusCode::CREATED,
        AppendHeaders([
            (header::SET_COOKIE, session),
            (header::SET_COOKIE, set_cookie(SIGNUP_COOKIE, "", 0)?),
        ]),
        Json(load_me(&state, user_id).await?),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_cookie_roundtrip_and_tamper() {
        let exp = Utc::now().timestamp() + 60;
        let v = pack(42, "카카오곰", exp);
        assert_eq!(unpack(&v), Some((42, "카카오곰".to_owned())));
        // 다른 사용자 id로 바꾸면 서명이 안 맞는다
        assert_eq!(unpack(&v.replacen("42.", "43.", 1)), None);
        assert_eq!(unpack(&pack(42, "a", Utc::now().timestamp() - 1)), None);
        assert_eq!(unpack("garbage"), None);
    }

    #[test]
    fn nickname_candidate_fits_rules() {
        assert_eq!(
            candidate(Some("카카오 곰👍 Bear123456789")),
            "카카오곰Bear1234"
        );
        assert_eq!(candidate(None), "");
    }
}
