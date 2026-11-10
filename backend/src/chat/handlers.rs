use std::collections::BTreeMap;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::Deserialize;

use super::db::{self, Row};
use super::{Chat, ChatBodyRequest, ChatError, ChatPage, Limits, ensure_can_chat};
use crate::AppState;
use crate::auth::contains_banned_word;
use crate::error::AppError;
use crate::extract::{CurrentUser, JsonBody};
use crate::realtime::{Event, WsEvent};

const DEFAULT_LIMIT: i64 = 50;
const MAX_LIMIT: i64 = 100;
const MAX_BODY_CHARS: usize = 200;

fn field_error(field: &str, reason: &str) -> AppError {
    AppError::Validation(BTreeMap::from([(field.to_owned(), reason.to_owned())]))
}

/// 앞뒤 공백을 자르고 1~200자·금지어를 확인한다. 욕설은 저장(원문)하고 응답에서 가린다.
fn check_body(body: &str) -> Result<String, AppError> {
    let body = body.trim();
    if !(1..=MAX_BODY_CHARS).contains(&body.chars().count()) {
        return Err(field_error("body", "1~200자로 입력해 주세요."));
    }
    if contains_banned_word(body) {
        return Err(ChatError::BannedWord.into());
    }
    Ok(body.to_owned())
}

#[derive(Deserialize)]
pub struct ListQuery {
    before: Option<String>,
    limit: Option<String>,
}

pub async fn list(
    State(state): State<Arc<AppState>>,
    Path(broadcast_id): Path<i64>,
    Query(q): Query<ListQuery>,
) -> Result<Json<ChatPage>, AppError> {
    let before = q
        .before
        .as_deref()
        .map(|s| {
            s.parse::<i64>()
                .map_err(|_| field_error("before", "올바르지 않은 커서예요."))
        })
        .transpose()?;
    let limit = match q.limit.as_deref().map(str::parse::<i64>) {
        None => DEFAULT_LIMIT,
        Some(Ok(n)) => n.clamp(1, MAX_LIMIT),
        Some(Err(_)) => return Err(field_error("limit", "숫자로 입력해 주세요.")),
    };
    db::broadcast_info(&state.db, broadcast_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let mut rows = db::list(&state.db, broadcast_id, before, limit + 1).await?;
    let has_more = rows.len() as i64 > limit;
    rows.truncate(limit as usize);
    rows.reverse();
    Ok(Json(ChatPage {
        items: rows.into_iter().map(Row::into_chat).collect(),
        has_more,
    }))
}

pub async fn create(
    State(state): State<Arc<AppState>>,
    Extension(limits): Extension<Arc<Limits>>,
    user: CurrentUser,
    Path(broadcast_id): Path<i64>,
    JsonBody(req): JsonBody<ChatBodyRequest>,
) -> Result<(StatusCode, Json<Chat>), AppError> {
    let (streamer_id, status) = db::broadcast_info(&state.db, broadcast_id)
        .await?
        .ok_or(AppError::NotFound)?;
    if status != "LIVE" {
        return Err(ChatError::NotLive.into());
    }
    ensure_can_chat(&state.db, streamer_id, user.id).await?;
    let body = check_body(&req.body)?;
    limits.hit(user.id, broadcast_id)?;
    if db::recent_same_body(&state.db, broadcast_id, user.id, &body).await? >= 2 {
        return Err(ChatError::Repeated.into());
    }

    let id = db::insert_if_live(&state.db, broadcast_id, user.id, &body)
        .await?
        .ok_or(ChatError::NotLive)?;
    let chat = db::find(&state.db, id)
        .await?
        .ok_or(AppError::NotFound)?
        .into_chat();
    state.realtime.publish(
        broadcast_id,
        Event::public(WsEvent::ChatCreated(chat.clone())),
    );
    Ok((StatusCode::CREATED, Json(chat)))
}

pub async fn update(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path(id): Path<i64>,
    JsonBody(req): JsonBody<ChatBodyRequest>,
) -> Result<Json<Chat>, AppError> {
    let row = db::find(&state.db, id).await?.ok_or(AppError::NotFound)?;
    if row.user_id != user.id {
        return Err(ChatError::NotAuthor.into());
    }
    if row.deleted_at.is_some() {
        return Err(ChatError::Deleted.into());
    }
    if row.broadcast_status != "LIVE" {
        return Err(ChatError::NotLive.into());
    }
    ensure_can_chat(&state.db, row.streamer_id, user.id).await?;
    let body = check_body(&req.body)?;
    if !db::update_body(&state.db, id, &body).await? {
        return Err(ChatError::EditExpired.into());
    }
    let chat = db::find(&state.db, id)
        .await?
        .ok_or(AppError::NotFound)?
        .into_chat();
    state.realtime.publish(
        chat.broadcast_id,
        Event::public(WsEvent::ChatUpdated(chat.clone())),
    );
    Ok(Json(chat))
}

pub async fn delete(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let row = db::find(&state.db, id).await?.ok_or(AppError::NotFound)?;
    if row.user_id != user.id && row.streamer_id != user.id {
        return Err(ChatError::DeleteForbidden.into());
    }
    if db::mark_deleted(&state.db, id).await? {
        state
            .realtime
            .publish(row.broadcast_id, Event::public(WsEvent::ChatDeleted { id }));
    }
    Ok(StatusCode::NO_CONTENT)
}
