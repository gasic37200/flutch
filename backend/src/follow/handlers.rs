use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};

use super::{UserProfile, db};
use crate::AppState;
use crate::broadcast::live_of_owner;
use crate::error::AppError;
use crate::extract::OptionalUser;

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
