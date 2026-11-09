//! Clutch Time 도메인 — docs/features/07-clutch.md

use std::sync::Arc;

use axum::Router;

use crate::AppState;
use crate::scheduler::Scheduler;

mod db;
mod handlers;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
}

/// 주기 작업: SUPER CLUTCH 회전(1초)
pub fn register(_s: &mut Scheduler) {}
