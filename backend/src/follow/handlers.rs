use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use serde::Deserialize;

use super::{FollowState, Pulse, PulseOffline, UserProfile, db};
use crate::AppState;
use crate::auth::UserSummary;
use crate::broadcast::{live_of_owner, live_of_owners};
use crate::error::AppError;
use crate::extract::{CurrentUser, OptionalUser};

pub async fn get_profile(
    State(state): State<Arc<AppState>>,
    OptionalUser(me): OptionalUser,
    Path(id): Path<i64>,
) -> Result<Json<UserProfile>, AppError> {
    let user = db::active_user(&state.db, id)
        .await?
        .ok_or(AppError::NotFound)?;
    let following = match me {
        Some(me) => Some(db::is_following(&state.db, me.id, id).await?),
        None => None,
    };
    Ok(Json(UserProfile {
        follower_count: db::follower_count(&state.db, id).await?,
        following,
        live_broadcast: live_of_owner(&state, id).await?,
        user,
    }))
}

async fn state_of(
    state: &AppState,
    streamer: i64,
    following: bool,
) -> Result<Json<FollowState>, AppError> {
    Ok(Json(FollowState {
        following,
        follower_count: db::follower_count(&state.db, streamer).await?,
    }))
}

pub async fn follow(
    State(state): State<Arc<AppState>>,
    me: CurrentUser,
    Path(id): Path<i64>,
) -> Result<Json<FollowState>, AppError> {
    if id == me.id {
        return Err(AppError::domain(
            StatusCode::UNPROCESSABLE_ENTITY,
            "FOLLOW_SELF",
            "자기 자신은 팔로우할 수 없어요.",
        ));
    }
    db::active_user(&state.db, id)
        .await?
        .ok_or(AppError::NotFound)?;
    db::follow(&state.db, me.id, id).await?;
    state_of(&state, id, true).await
}

pub async fn unfollow(
    State(state): State<Arc<AppState>>,
    me: CurrentUser,
    Path(id): Path<i64>,
) -> Result<Json<FollowState>, AppError> {
    db::active_user(&state.db, id)
        .await?
        .ok_or(AppError::NotFound)?;
    db::unfollow(&state.db, me.id, id).await?;
    state_of(&state, id, false).await
}

#[derive(Deserialize)]
pub struct PulseQuery {
    limit: Option<i64>,
}

pub async fn pulse(
    State(state): State<Arc<AppState>>,
    me: CurrentUser,
    Query(q): Query<PulseQuery>,
) -> Result<Json<Pulse>, AppError> {
    let limit = match q.limit {
        None => 10,
        Some(n @ 1..=50) => n,
        Some(_) => {
            return Err(AppError::Validation(
                [("limit".to_string(), "1~50 사이여야 해요.".to_string())].into(),
            ));
        }
    } as usize;
    let followed = db::followed(&state.db, me.id).await?;
    let total = followed.len() as i64;
    let ids: Vec<i64> = followed.iter().map(|f| f.id).collect();
    let mut live = live_of_owners(&state, &ids).await?;
    live.sort_by_key(|b| std::cmp::Reverse((b.viewer_count, b.id)));
    live.truncate(limit);
    let live_ids: Vec<i64> = live.iter().map(|b| b.owner.id).collect();
    let offline = followed
        .into_iter()
        .filter(|f| !live_ids.contains(&f.id))
        .take(limit - live.len())
        .map(|f| PulseOffline {
            user: UserSummary {
                id: f.id,
                nickname: f.nickname,
            },
            follower_count: f.follower_count,
            last_ended_at: f.last_ended_at,
        })
        .collect();
    Ok(Json(Pulse {
        live,
        offline,
        total,
    }))
}
