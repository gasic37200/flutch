//! 팔로우 도메인 — docs/features/09-follow.md

use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use serde::Serialize;
use ts_rs::TS;

use crate::AppState;
use crate::auth::UserSummary;
use crate::broadcast::Broadcast;

mod db;
mod handlers;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/users/{id}", get(handlers::get_profile))
}

/// `GET /users/{id}` 응답 (api.md §9).
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UserProfile {
    pub user: UserSummary,
    pub follower_count: i64,
    /// 비로그인은 null
    pub following: Option<bool>,
    pub live_broadcast: Option<Broadcast>,
}
