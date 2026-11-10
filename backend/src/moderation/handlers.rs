use std::collections::BTreeMap;
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use serde::Deserialize;

use super::{Ban, BanRequest, ModerationError, SystemKind, db};
use crate::AppState;
use crate::auth::UserSummary;
use crate::broadcast::Page;
use crate::chat::ChatPage;
use crate::error::AppError;
use crate::extract::{CurrentUser, JsonBody};
use crate::realtime::{Event, WsEvent};

const DEFAULT_LIMIT: i64 = 50;
const MAX_LIMIT: i64 = 100;

#[derive(Deserialize)]
pub struct ListQuery {
    cursor: Option<String>,
    before: Option<String>,
    limit: Option<String>,
}

fn bad(field: &str, msg: &str) -> AppError {
    AppError::Validation(BTreeMap::from([(field.to_owned(), msg.to_owned())]))
}

fn num(field: &str, v: &Option<String>) -> Result<Option<i64>, AppError> {
    v.as_deref()
        .map(|s| s.parse().map_err(|_| bad(field, "올바르지 않은 값이에요.")))
        .transpose()
}

fn limit_of(q: &ListQuery) -> Result<i64, AppError> {
    Ok(num("limit", &q.limit)?
        .unwrap_or(DEFAULT_LIMIT)
        .clamp(1, MAX_LIMIT))
}

fn ensure_owner(user: &CurrentUser, id: i64) -> Result<(), AppError> {
    if user.id == id {
        Ok(())
    } else {
        Err(ModerationError::NotOwner.into())
    }
}

pub async fn list(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path(id): Path<i64>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Page<Ban>>, AppError> {
    ensure_owner(&user, id)?;
    let limit = limit_of(&q)?;
    let mut rows = db::list(&state.db, id, num("cursor", &q.cursor)?, limit + 1).await?;
    // 한 건 더 읽어 다음 페이지 유무를 안다
    let next_cursor = if rows.len() as i64 > limit {
        rows.truncate(limit as usize);
        rows.last().map(|r| r.id.to_string())
    } else {
        None
    };
    let items = rows
        .into_iter()
        .map(|r| Ban {
            user: UserSummary {
                id: r.user_id,
                nickname: r.nickname,
            },
            permanent: r.until.is_none(),
            until: r.until,
            created_at: r.created_at,
        })
        .collect();
    Ok(Json(Page { items, next_cursor }))
}

pub async fn put(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path((id, target)): Path<(i64, i64)>,
    JsonBody(req): JsonBody<BanRequest>,
) -> Result<Json<Ban>, AppError> {
    ensure_owner(&user, id)?;
    if target == id {
        return Err(ModerationError::SelfBan.into());
    }
    let nickname = db::nickname(&state.db, target)
        .await?
        .ok_or(AppError::NotFound)?;
    let until = req.duration.until();
    let (created_at, live) = db::upsert(&state.db, id, target, until).await?;
    let target_user = UserSummary {
        id: target,
        nickname,
    };
    if let Some(broadcast_id) = live {
        let kind = if until.is_some() {
            SystemKind::Timeout
        } else {
            SystemKind::Ban
        };
        state.realtime.publish(
            broadcast_id,
            Event::public(WsEvent::ChatSystem {
                kind,
                user: target_user.clone(),
                until,
            }),
        );
    }
    Ok(Json(Ban {
        user: target_user,
        permanent: until.is_none(),
        until,
        created_at,
    }))
}

pub async fn delete(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path((id, target)): Path<(i64, i64)>,
) -> Result<StatusCode, AppError> {
    ensure_owner(&user, id)?;
    db::remove(&state.db, id, target).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn viewer_chats(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path((id, target)): Path<(i64, i64)>,
    Query(q): Query<ListQuery>,
) -> Result<Json<ChatPage>, AppError> {
    ensure_owner(&user, id)?;
    let limit = limit_of(&q)?;
    db::nickname(&state.db, target)
        .await?
        .ok_or(AppError::NotFound)?;
    let mut rows =
        db::viewer_chats(&state.db, id, target, num("before", &q.before)?, limit + 1).await?;
    let has_more = rows.len() as i64 > limit;
    rows.truncate(limit as usize);
    rows.reverse();
    Ok(Json(ChatPage {
        items: rows.into_iter().map(|r| r.into_chat()).collect(),
        has_more,
    }))
}
