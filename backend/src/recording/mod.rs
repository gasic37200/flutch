//! 녹화·다시보기 도메인 — docs/features/08-recording.md

use std::sync::Arc;

use axum::Router;

use crate::AppState;
use crate::scheduler::Scheduler;

mod db;
mod handlers;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
}

/// 주기 작업: 보관 정리(1시간)
pub fn register(_s: &mut Scheduler) {}
