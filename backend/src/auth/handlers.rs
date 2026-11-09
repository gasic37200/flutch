use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration as StdDuration;

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{AppendHeaders, IntoResponse};
use axum::{Extension, Json};
use chrono::{DateTime, Duration, Utc};

use super::rate_limit::{RateLimiter, client_ip};
use super::{
    AuthError, ChangePasswordRequest, LoginRequest, Me, SEED_ACCOUNTS, SignupRequest,
    TestLoginRequest, UpdateMeRequest, db, rules, session,
};
use crate::AppState;
use crate::error::AppError;
use crate::extract::{CurrentUser, JsonBody};

const NICKNAME_COOLDOWN_DAYS: i64 = 30;

/// IP별 요청 제한(api.md §2): 가입 1시간 5회, 로그인 1분 20회.
pub struct Limits {
    signup: RateLimiter,
    login: RateLimiter,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            signup: RateLimiter::new(5, StdDuration::from_secs(60 * 60)),
            login: RateLimiter::new(20, StdDuration::from_secs(60)),
        }
    }
}

type Fields = BTreeMap<String, String>;

fn check(fields: &mut Fields, name: &str, result: Result<(), &'static str>) {
    if let Err(reason) = result {
        fields.insert(name.to_owned(), reason.to_owned());
    }
}

fn require(fields: &mut Fields, name: &str, ok: bool, reason: &'static str) {
    check(fields, name, if ok { Ok(()) } else { Err(reason) });
}

fn validated(fields: Fields) -> Result<(), AppError> {
    if fields.is_empty() {
        Ok(())
    } else {
        Err(AppError::Validation(fields))
    }
}

fn remaining_secs(until: Option<DateTime<Utc>>) -> Option<i64> {
    let left = (until? - Utc::now()).num_seconds();
    (left > 0).then_some(left)
}

async fn load_me(state: &AppState, user_id: i64) -> Result<Me, AppError> {
    let row = db::me(&state.db, user_id).await?;
    Ok(Me {
        id: row.id,
        login_id: row.login_id,
        nickname: row.nickname,
        nickname_changeable_at: row
            .nickname_changed_at
            .map(|at| at + Duration::days(NICKNAME_COOLDOWN_DAYS))
            .filter(|at| *at > Utc::now()),
        has_password: row.has_password,
        balance: row.balance,
        live_broadcast_id: row.live_broadcast_id,
    })
}

pub async fn signup(
    State(state): State<Arc<AppState>>,
    Extension(limits): Extension<Arc<Limits>>,
    headers: HeaderMap,
    JsonBody(req): JsonBody<SignupRequest>,
) -> Result<impl IntoResponse, AppError> {
    let login_id = req.login_id.trim().to_lowercase();
    let nickname = req.nickname.trim().to_owned();
    let mut fields = Fields::new();
    check(&mut fields, "loginId", rules::check_login_id(&login_id));
    check(
        &mut fields,
        "password",
        rules::check_password(&req.password, &login_id),
    );
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

    // 입력 실수(422)는 세지 않고 가입 시도만 센다.
    if let Err(e) = limits.signup.hit(&client_ip(&headers)) {
        tracing::warn!("signup rate limited");
        return Err(e);
    }
    if db::login_id_exists(&state.db, &login_id).await? {
        return Err(AuthError::LoginIdTaken.into());
    }
    if db::nickname_taken(&state.db, &nickname, None).await? {
        return Err(AuthError::NicknameTaken.into());
    }
    let hash = session::hash_password(req.password).await?;
    let user_id = db::insert_user(&state.db, &login_id, &hash, &nickname).await?;
    tracing::info!(user_id, "user signed up");
    let cookie = session::start(&state.db, user_id, false).await?;
    Ok((
        StatusCode::CREATED,
        [(header::SET_COOKIE, cookie)],
        Json(load_me(&state, user_id).await?),
    ))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Extension(limits): Extension<Arc<Limits>>,
    headers: HeaderMap,
    JsonBody(req): JsonBody<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    if let Err(e) = limits.login.hit(&client_ip(&headers)) {
        tracing::warn!("login rate limited");
        return Err(e);
    }
    let login_id = req.login_id.trim().to_lowercase();
    // 없는 아이디·탈퇴 계정·카카오 계정 모두 같은 문구.
    let Some(user) = db::find_login(&state.db, &login_id).await? else {
        return Err(AuthError::InvalidCredentials.into());
    };
    let Some(hash) = user.password_hash else {
        return Err(AuthError::InvalidCredentials.into());
    };
    if let Some(left) = remaining_secs(user.locked_until) {
        tracing::warn!(user_id = user.id, "login rejected: locked");
        return Err(AuthError::Locked(left).into());
    }
    if !session::verify_password(req.password, hash).await? {
        let locked_until = db::record_login_failure(&state.db, user.id).await?;
        if let Some(left) = remaining_secs(locked_until) {
            tracing::warn!(user_id = user.id, "login locked after repeated failures");
            return Err(AuthError::Locked(left).into());
        }
        return Err(AuthError::InvalidCredentials.into());
    }
    db::reset_login_failures(&state.db, user.id).await?;
    let cookie = session::start(&state.db, user.id, req.remember).await?;
    Ok((
        [(header::SET_COOKIE, cookie)],
        Json(load_me(&state, user.id).await?),
    ))
}

pub async fn test_login(
    State(state): State<Arc<AppState>>,
    JsonBody(req): JsonBody<TestLoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let Some((login_id, _)) = SEED_ACCOUNTS.iter().find(|(id, _)| *id == req.account) else {
        let mut fields = Fields::new();
        fields.insert("account".into(), "테스트 계정을 골라 주세요.".into());
        return Err(AppError::Validation(fields));
    };
    super::ensure_seed_accounts(&state).await?;
    let user_id = db::find_active_id(&state.db, login_id)
        .await?
        .ok_or_else(|| AppError::Internal(format!("seed account {login_id} missing")))?;
    let cookie = session::start(&state.db, user_id, false).await?;
    Ok((
        [(header::SET_COOKIE, cookie)],
        Json(load_me(&state, user_id).await?),
    ))
}

pub async fn logout(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    if let Some(token) = session::token(&headers) {
        db::delete_session(&state.db, &session::hash(token)).await?;
    }
    Ok((
        StatusCode::NO_CONTENT,
        [(header::SET_COOKIE, session::cookie("", Duration::zero())?)],
    ))
}

pub async fn me(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    // 쿠키 만료도 DB 세션 연장에 맞춰 다시 늘린다("사용 시 연장"). 화면은 시작할 때 /me를 부른다.
    let cookie = session::token(&headers)
        .map(|t| session::cookie(t, session::lifetime(user.remember)))
        .transpose()?;
    Ok((
        AppendHeaders(cookie.map(|c| (header::SET_COOKIE, c))),
        Json(load_me(&state, user.id).await?),
    ))
}

pub async fn update_me(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    JsonBody(req): JsonBody<UpdateMeRequest>,
) -> Result<Json<Me>, AppError> {
    let nickname = req.nickname.trim().to_owned();
    let mut fields = Fields::new();
    check(&mut fields, "nickname", rules::check_nickname(&nickname));
    validated(fields)?;

    let current = load_me(&state, user.id).await?;
    if current.nickname == nickname {
        return Ok(Json(current));
    }
    if let Some(at) = current.nickname_changeable_at {
        let days = ((at - Utc::now()).num_seconds() + 86_399) / 86_400;
        return Err(AuthError::NicknameChangeTooSoon(days.max(1)).into());
    }
    if db::nickname_taken(&state.db, &nickname, Some(user.id)).await? {
        return Err(AuthError::NicknameTaken.into());
    }
    db::update_nickname(&state.db, user.id, &nickname).await?;
    Ok(Json(load_me(&state, user.id).await?))
}

pub async fn change_password(
    State(state): State<Arc<AppState>>,
    Extension(limits): Extension<Arc<Limits>>,
    user: CurrentUser,
    JsonBody(req): JsonBody<ChangePasswordRequest>,
) -> Result<StatusCode, AppError> {
    // 탈취한 세션으로 현재 비밀번호를 무한 대입하지 못하게 사용자별로 로그인과 같은 한도를 건다.
    limits.login.hit(&format!("pw:{}", user.id))?;
    let creds = db::credentials(&state.db, user.id).await?;
    // 카카오 계정은 비밀번호가 없어 바꿀 수 없다.
    let (Some(login_id), Some(hash)) = (creds.login_id, creds.password_hash) else {
        return Err(AppError::Forbidden);
    };
    let mut fields = Fields::new();
    check(
        &mut fields,
        "newPassword",
        rules::check_password(&req.new_password, &login_id),
    );
    validated(fields)?;
    if !session::verify_password(req.current_password, hash).await? {
        return Err(AuthError::WrongPassword.into());
    }
    let new_hash = session::hash_password(req.new_password).await?;
    db::change_password(&state.db, user.id, &new_hash, &user.session_hash).await?;
    tracing::info!(
        user_id = user.id,
        "password changed, other sessions revoked"
    );
    Ok(StatusCode::NO_CONTENT)
}
