//! 송출·시청 도메인 — docs/features/03-streaming.md

use std::sync::Arc;

use axum::Router;
use axum::http::StatusCode;
use axum::routing::post;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::AppState;
use crate::broadcast::BroadcastDetail;
use crate::error::AppError;
use crate::scheduler::Scheduler;

mod db;
mod handlers;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/broadcasts/{id}/live",
            post(handlers::start).delete(handlers::end),
        )
        .route(
            "/broadcasts/{id}/viewer-token",
            post(handlers::viewer_token),
        )
}

/// 주기 작업: 유예·3시간 자동 종료(5초)
pub fn register(_s: &mut Scheduler) {}

/// LiveKit 접속 정보. `url`은 설정값 `LIVEKIT_URL`.
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LiveKitToken {
    pub token: String,
    pub url: String,
    pub identity: String,
    pub room: String,
}

#[derive(Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StartLiveRequest {
    pub tab_id: String,
    /// 다른 기기가 송출 중일 때 그 기기를 끊고 이 기기로 전환
    #[ts(optional)]
    pub takeover: Option<bool>,
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StartLiveResponse {
    pub broadcast: BroadcastDetail,
    pub livekit: LiveKitToken,
}

#[derive(Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ViewerTokenRequest {
    pub tab_id: String,
    /// 비로그인 필수, 로그인 사용자는 무시
    #[ts(optional)]
    pub guest_id: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum LiveError {
    #[error("다른 기기에서 송출 중이에요. 이 기기로 전환할까요?")]
    PublisherActive,
    #[error("방송 중일 때만 할 수 있어요.")]
    NotLive,
}

impl From<LiveError> for AppError {
    fn from(e: LiveError) -> Self {
        let code = match e {
            LiveError::PublisherActive => "LIVE_PUBLISHER_ACTIVE",
            LiveError::NotLive => "BROADCAST_NOT_LIVE",
        };
        AppError::domain(StatusCode::CONFLICT, code, e.to_string())
    }
}
