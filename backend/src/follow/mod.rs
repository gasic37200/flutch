//! 팔로우 도메인 — docs/features/09-follow.md

use std::sync::Arc;

use axum::Router;

use crate::AppState;

mod db;
mod handlers;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
}
