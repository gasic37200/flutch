//! 송출·시청 도메인 — docs/features/03-streaming.md

use std::sync::Arc;

use axum::Router;

use crate::AppState;
use crate::scheduler::Scheduler;

mod db;
mod handlers;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
}

/// 주기 작업: 유예·3시간 자동 종료(5초)
pub fn register(_s: &mut Scheduler) {}
