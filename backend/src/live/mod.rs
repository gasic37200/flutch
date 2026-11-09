//! 송출·시청 도메인 — docs/features/03-streaming.md

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

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
mod jobs;
mod webhook;

pub use jobs::{check_limits, sync};

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
        .route("/livekit/webhook", post(webhook::receive))
}

/// 주기 작업(5초): 시작 후 첫 성공 한 번은 LiveKit 실제 상태와 동기화(실패하면 다음 주기에 다시), 매번 유예·3시간 자동 종료.
pub fn register(s: &mut Scheduler) {
    s.every("live", Duration::from_secs(5), |state| async move {
        if !state.live.synced.load(Ordering::Relaxed) {
            match sync(&state).await {
                Ok(()) => state.live.synced.store(true, Ordering::Relaxed),
                Err(e) => tracing::error!(error = %e, "livekit sync failed, retrying next tick"),
            }
        }
        check_limits(&state).await
    });
}

/// 송출 메모리 상태. 재시작하면 비고 `sync`가 LiveKit 참가자 목록으로 복구한다.
#[derive(Default)]
pub struct LiveState {
    /// 방송별 시청자 LiveKit 신원(소유자·송출자·Egress 제외)
    viewers: Mutex<HashMap<i64, HashSet<String>>>,
    /// 자동 종료 경고를 보낸 방송. shortcut: 메모리라 재시작 후 경고 구간이면 한 번 더 보낸다, 문제 되면 DB 컬럼으로.
    warned: Mutex<HashSet<i64>>,
    synced: AtomicBool,
}

/// 잠금 중 패닉이 나도 맵은 일관적이다(한 번의 삽입·삭제뿐).
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

/// 신원 `u:<id>:<tab>`/`g:<guest>:<tab>`을 사용자 단위로 묶은 수(같은 사용자 여러 탭은 1명).
fn user_count(identities: &HashSet<String>) -> i64 {
    let users: HashSet<_> = identities
        .iter()
        .filter_map(|i| i.rsplit_once(':'))
        .map(|(user, _tab)| user)
        .collect();
    users.len() as i64
}

/// 시청자로 셀 신원인가: 우리 토큰(`u:`/`g:`)이고 방송 소유자가 아님.
fn is_viewer(identity: &str, owner_id: i64) -> bool {
    (identity.starts_with("u:") || identity.starts_with("g:"))
        && !identity.starts_with(&format!("u:{owner_id}:"))
}

impl LiveState {
    /// 방송의 현재 시청자 수(메모리). 없으면 0.
    pub fn viewer_count(&self, id: i64) -> i64 {
        lock(&self.viewers).get(&id).map_or(0, user_count)
    }

    /// 방송의 신원 집합을 바꾸고, 시청자 수가 달라졌으면 새 값.
    fn update_viewers(&self, id: i64, f: impl FnOnce(&mut HashSet<String>)) -> Option<i64> {
        let mut map = lock(&self.viewers);
        let set = map.entry(id).or_default();
        let before = user_count(set);
        f(set);
        let after = user_count(set);
        if set.is_empty() {
            map.remove(&id);
        }
        (after != before).then_some(after)
    }
}

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
    #[error("웹훅 서명이 올바르지 않아요.")]
    WebhookSignature,
}

impl From<LiveError> for AppError {
    fn from(e: LiveError) -> Self {
        let (status, code) = match e {
            LiveError::PublisherActive => (StatusCode::CONFLICT, "LIVE_PUBLISHER_ACTIVE"),
            LiveError::NotLive => (StatusCode::CONFLICT, "BROADCAST_NOT_LIVE"),
            LiveError::WebhookSignature => {
                (StatusCode::UNAUTHORIZED, "LIVE_WEBHOOK_INVALID_SIGNATURE")
            }
        };
        AppError::domain(status, code, e.to_string())
    }
}
