//! Clutch Time 도메인 — docs/features/07-clutch.md

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Extension, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, PgPool};
use ts_rs::TS;

use crate::AppState;
use crate::auth::rate_limit::RateLimiter;
use crate::config::ClutchConfig;
use crate::error::AppError;
use crate::realtime::{Event, WsEvent};
use crate::scheduler::Scheduler;
use crate::wallet::Donation;

mod db;
mod handlers;
mod rotation;

pub use rotation::rotate;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/broadcasts/{id}/clutches", post(handlers::create))
        .route("/broadcasts/{id}/bursts", post(handlers::burst))
        .route("/home/hero", get(rotation::hero))
        .route("/clips", get(handlers::clips))
        .layer(Extension(Arc::new(Limits::default())))
}

/// 주기 작업: SUPER CLUTCH 회전(1초).
pub fn register(s: &mut Scheduler) {
    s.every("clutch_rotation", Duration::from_secs(1), rotate);
}

/// 사용자·방송 단위 초당 10회(메모리라 서버를 다시 띄우면 비워진다).
pub(crate) struct Limits(RateLimiter);

impl Default for Limits {
    fn default() -> Self {
        Self(RateLimiter::new(10, Duration::from_secs(1)))
    }
}

impl Limits {
    fn hit(&self, user_id: i64, broadcast_id: i64) -> Result<(), AppError> {
        self.0.hit(&format!("{user_id}:{broadcast_id}"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ClutchPhase {
    Charging,
    Imminent,
    Cooldown,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ClutchGauge {
    /// 0~100 정수
    pub percent: i32,
    pub phase: ClutchPhase,
    pub cooldown_until: Option<DateTime<Utc>>,
    /// 100%까지 남은 비트(SUPER BURST 1비트 = 1점). 쿨다운 중에도 같은 계산이지만 채워도 쿨다운이 끝나야 발동한다.
    pub bits_to_full: i64,
}

/// SUPER CLUTCH 발동(`clutch.super` 이벤트)
#[derive(Debug, Clone, Copy, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SuperClutch {
    pub triggered_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// `POST /broadcasts/{id}/clutches` 응답
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ClutchResult {
    pub paid: bool,
    /// 무료 1~5, 유료 0
    pub combo: i32,
    pub free_clutches_left: i32,
    /// 유료일 때만
    pub balance: Option<i64>,
    pub gauge: ClutchGauge,
}

/// `POST /broadcasts/{id}/bursts` 요청
#[derive(Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct BurstRequest {
    /// 쓸 비트 = 게이지 점수. `burstMin` ~ 1,000,000
    pub amount: i64,
    pub anonymous: bool,
}

/// `POST /broadcasts/{id}/bursts` 응답
#[derive(Serialize, TS)]
#[ts(export)]
pub struct BurstResult {
    /// 내 버스트라 실명
    pub donation: Donation,
    pub balance: i64,
    pub gauge: ClutchGauge,
}

#[derive(Debug, thiserror::Error)]
pub enum ClutchError {
    #[error("내 방송에는 클러치할 수 없어요.")]
    SelfClutch,
    #[error("방송 중일 때만 할 수 있어요.")]
    NotLive,
    #[error("이 채널에서는 유료 클러치를 할 수 없어요.")]
    Banned,
}

impl From<ClutchError> for AppError {
    fn from(e: ClutchError) -> Self {
        let (status, code) = match e {
            ClutchError::SelfClutch => (StatusCode::FORBIDDEN, "CLUTCH_SELF"),
            ClutchError::NotLive => (StatusCode::CONFLICT, "BROADCAST_NOT_LIVE"),
            ClutchError::Banned => (StatusCode::FORBIDDEN, "MODERATION_BANNED"),
        };
        AppError::domain(status, code, e.to_string())
    }
}

/// 게이지에 점수를 넣는 출처(`clutch_events.source`)
#[derive(Debug, Clone, Copy)]
pub enum Source {
    Free,
    Paid,
    Burst,
}

impl Source {
    fn as_str(self) -> &'static str {
        match self {
            Self::Free => "free",
            Self::Paid => "paid",
            Self::Burst => "burst",
        }
    }
}

/// `add_points` 결과. 커밋 뒤 `publish`에 넘긴다.
pub struct Outcome {
    pub gauge: ClutchGauge,
    /// 이 호출이 SUPER CLUTCH를 발동시켰으면 그 정보
    pub super_clutch: Option<SuperClutch>,
}

/// LIVE가 아닌 방송의 게이지
pub fn idle_gauge(cfg: &ClutchConfig) -> ClutchGauge {
    build_gauge(0, None, Utc::now(), cfg)
}

fn build_gauge(
    points: i64,
    cooldown_until: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
    cfg: &ClutchConfig,
) -> ClutchGauge {
    let active = cooldown_until.filter(|t| *t > now);
    let percent = (points.saturating_mul(100) / cfg.gauge_target).clamp(0, 100) as i32;
    let phase = if active.is_some() {
        ClutchPhase::Cooldown
    } else if percent >= cfg.imminent_percent {
        ClutchPhase::Imminent
    } else {
        ClutchPhase::Charging
    };
    ClutchGauge {
        percent,
        phase,
        cooldown_until: active,
        bits_to_full: (cfg.gauge_target - points).max(0),
    }
}

/// 마지막 발동 기준 재발동 가능 시각: 진행 중이 끝난 뒤와 발동 후 쿨다운 중 늦은 쪽.
fn cooldown_of(
    triggered: DateTime<Utc>,
    expires: DateTime<Utc>,
    cfg: &ClutchConfig,
) -> DateTime<Utc> {
    expires.max(triggered + chrono::Duration::seconds(cfg.cooldown_secs))
}

/// 현재 게이지(마지막 SUPER CLUTCH 이후 최근 창 안의 점수 합).
pub async fn gauge_of(
    db: &PgPool,
    cfg: &ClutchConfig,
    broadcast_id: i64,
) -> Result<ClutchGauge, sqlx::Error> {
    let g = db::gauge_row(db, broadcast_id, cfg.window_secs).await?;
    let cooldown = g.last.map(|(t, e)| cooldown_of(t, e, cfg));
    Ok(build_gauge(g.points, cooldown, g.now, cfg))
}

/// 이 사용자가 이 방송에서 지금 쓸 수 있는 무료 클러치(0~5).
pub async fn free_left(
    db: &PgPool,
    cfg: &ClutchConfig,
    user_id: i64,
    broadcast_id: i64,
) -> Result<i32, sqlx::Error> {
    let used = db::free_used(db, user_id, broadcast_id, cfg.free_window_secs).await?;
    Ok((cfg.free_per_window - used).max(0) as i32)
}

/// 게이지 점수를 넣고 SUPER CLUTCH 발동을 판정한다. 호출자의 트랜잭션 안에서 부르고 **커밋 뒤** `publish`한다.
/// 방송별 트랜잭션 advisory lock으로 직렬화해서 동시에 100%를 넘겨도 발동은 한 번만 한다.
/// 버스트는 쓴 비트를 그대로 `points`로 넘긴다(1비트 = 1점).
pub async fn add_points(
    conn: &mut PgConnection,
    cfg: &ClutchConfig,
    broadcast_id: i64,
    user_id: i64,
    source: Source,
    points: i64,
) -> Result<Outcome, sqlx::Error> {
    db::lock(conn, &format!("clutch:gauge:{broadcast_id}")).await?;
    db::insert_event(conn, broadcast_id, user_id, source.as_str(), points).await?;
    let g = db::gauge_row(&mut *conn, broadcast_id, cfg.window_secs).await?;
    let cooldown = g.last.map(|(t, e)| cooldown_of(t, e, cfg));
    let in_cooldown = cooldown.is_some_and(|t| t > g.now);
    if g.points >= cfg.gauge_target && !in_cooldown {
        let (triggered_at, expires_at) =
            db::insert_super(conn, broadcast_id, cfg.super_secs).await?;
        let cd = cooldown_of(triggered_at, expires_at, cfg);
        return Ok(Outcome {
            gauge: build_gauge(0, Some(cd), triggered_at, cfg),
            super_clutch: Some(SuperClutch {
                triggered_at,
                expires_at,
            }),
        });
    }
    Ok(Outcome {
        gauge: build_gauge(g.points, cooldown, g.now, cfg),
        super_clutch: None,
    })
}

/// 같은 방송의 `clutch.gauge` 최소 간격(초당 최대 2회)
const GAUGE_GAP: Duration = Duration::from_millis(500);

#[derive(Default)]
struct Slot {
    last: Option<Instant>,
    /// 간격 안에 들어온 가장 최근 게이지(타이머가 보낸다)
    pending: Option<ClutchGauge>,
}

/// 클러치 메모리 상태(`AppState.clutch`): 방송별 `clutch.gauge` 묶음.
// shortcut: 프로세스 메모리(서버 1대 가정). 서버를 여러 대로 늘리면 이벤트 허브와 함께 공유 저장소로 옮긴다.
#[derive(Default)]
pub struct ClutchState {
    slots: Mutex<HashMap<i64, Slot>>,
}

impl ClutchState {
    fn slots(&self) -> MutexGuard<'_, HashMap<i64, Slot>> {
        // 잠금 중 패닉이 나도 맵은 일관적이다(필드 대입뿐).
        self.slots.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// 커밋 뒤 호출: 발동이면 `clutch.super`를 바로, 게이지는 방송당 초당 2회로 묶어서 보낸다.
pub fn publish(state: &Arc<AppState>, broadcast_id: i64, o: &Outcome) {
    if let Some(s) = o.super_clutch {
        state
            .realtime
            .publish(broadcast_id, Event::public(WsEvent::ClutchSuper(s)));
    }
    publish_gauge(
        state,
        broadcast_id,
        o.gauge.clone(),
        o.super_clutch.is_some(),
    );
}

fn publish_gauge(state: &Arc<AppState>, id: i64, g: ClutchGauge, force: bool) {
    let now = Instant::now();
    let mut slots = state.clutch.slots();
    if slots.len() > 1000 {
        slots.retain(|_, s| {
            s.pending.is_some() || s.last.is_some_and(|l| now - l < Duration::from_secs(60))
        });
    }
    let slot = slots.entry(id).or_default();
    let wait = slot.last.map(|l| GAUGE_GAP.saturating_sub(now - l));
    if force || (slot.pending.is_none() && wait.is_none_or(|w| w.is_zero())) {
        slot.last = Some(now);
        slot.pending = None;
        drop(slots);
        state
            .realtime
            .publish(id, Event::public(WsEvent::ClutchGauge(g)));
        return;
    }
    let first = slot.pending.is_none();
    slot.pending = Some(g);
    drop(slots);
    if first {
        let state = state.clone();
        let wait = wait.unwrap_or(GAUGE_GAP);
        tokio::spawn(async move {
            tokio::time::sleep(wait).await;
            let mut slots = state.clutch.slots();
            let g = slots.get_mut(&id).and_then(|s| {
                s.last = Some(Instant::now());
                s.pending.take()
            });
            drop(slots);
            if let Some(g) = g {
                state
                    .realtime
                    .publish(id, Event::public(WsEvent::ClutchGauge(g)));
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gauge_phases() {
        let cfg = ClutchConfig::default();
        let now = Utc::now();
        assert_eq!(
            build_gauge(79_000, None, now, &cfg).phase,
            ClutchPhase::Charging
        );
        assert_eq!(
            build_gauge(80_000, None, now, &cfg).phase,
            ClutchPhase::Imminent
        );
        let cd = Some(now + chrono::Duration::minutes(1));
        let g = build_gauge(200_000, cd, now, &cfg);
        assert_eq!((g.percent, g.phase), (100, ClutchPhase::Cooldown));
        assert_eq!(g.bits_to_full, 0);
        // 99,999점은 99%, 남은 1비트
        let g = build_gauge(99_999, None, now, &cfg);
        assert_eq!((g.percent, g.bits_to_full), (99, 1));
        let later = now + chrono::Duration::minutes(2);
        assert_eq!(build_gauge(0, cd, later, &cfg).phase, ClutchPhase::Charging);
    }
}
