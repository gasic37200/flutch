//! 지갑·결제·후원 도메인 — docs/features/06-wallet-donation.md

use std::sync::Arc;

use axum::Router;

use crate::AppState;
use crate::scheduler::Scheduler;

mod db;
mod handlers;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
}

/// 주기 작업: 만료 결제 대기 주문 정리(1분)
pub fn register(_s: &mut Scheduler) {}
