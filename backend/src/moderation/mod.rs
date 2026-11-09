//! 모더레이션(밴) 도메인 — docs/features/05-moderation.md

use std::sync::Arc;

use axum::Router;

use crate::AppState;

mod db;
mod handlers;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
}
