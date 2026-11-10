//! 팔로우 도메인 — docs/features/09-follow.md

use std::sync::Arc;

use axum::Router;
use axum::routing::{get, put};
use chrono::{DateTime, Utc};
use serde::Serialize;
use ts_rs::TS;

use crate::AppState;
use crate::auth::UserSummary;
use crate::broadcast::Broadcast;

mod db;
mod handlers;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/users/{id}", get(handlers::get_profile))
        .route(
            "/users/{id}/follow",
            put(handlers::follow).delete(handlers::unfollow),
        )
        .route("/pulse", get(handlers::pulse))
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

/// `PUT·DELETE /users/{id}/follow` 응답.
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FollowState {
    pub following: bool,
    pub follower_count: i64,
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PulseOffline {
    pub user: UserSummary,
    pub follower_count: i64,
    pub last_ended_at: Option<DateTime<Utc>>,
}

/// `GET /pulse` 응답.
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Pulse {
    /// 시청자 수 순
    pub live: Vec<Broadcast>,
    pub offline: Vec<PulseOffline>,
    /// 팔로우한 스트리머 전체 수(limit 무관)
    pub total: i64,
}
