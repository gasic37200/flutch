//! 인증·계정 도메인 — docs/features/01-accounts.md

use std::sync::Arc;

use axum::Router;

use crate::AppState;
use crate::scheduler::Scheduler;

mod db;
mod handlers;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
}

/// 주기 작업: 만료 세션 정리(1일)
pub fn register(_s: &mut Scheduler) {}
