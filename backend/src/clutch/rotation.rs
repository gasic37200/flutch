//! 진행 중 SUPER CLUTCH 회전과 홈 히어로(features/07 "메인 히어로", api.md §7).
//! 상태는 `super_clutches.shown_at`(DB)뿐이라 서버를 다시 띄워도 같은 슬롯이 이어진다.
//! 회전 순서: 현재(가장 늦게 노출) → 아직 노출 전(`rotation_order`, 같으면 id) → 나머지(오래전에 노출된 순).
//! T19의 `rotation_order`는 방송 간 동시 발동이면 같은 값일 수 있어 id로 동률을 끊는다.

use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use sqlx::PgPool;
use ts_rs::TS;

use super::db::{self, Active};
use super::{ClutchGauge, gauge_of};
use crate::AppState;
use crate::broadcast::{self, Broadcast};
use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum HeroMode {
    SuperClutch,
    TopGauge,
    TopViewers,
    Empty,
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct HeroSlot {
    pub started_at: DateTime<Utc>,
    pub next_switch_at: DateTime<Utc>,
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct HeroActive {
    pub broadcast: Broadcast,
    pub expires_at: DateTime<Utc>,
}

/// `GET /home/hero` 응답
#[derive(Serialize, TS)]
#[ts(export)]
pub struct Hero {
    pub mode: HeroMode,
    /// 지금 히어로 방송(empty면 null)
    pub broadcast: Option<Broadcast>,
    pub gauge: Option<ClutchGauge>,
    /// super_clutch 회전 중일 때 현재 슬롯
    pub slot: Option<HeroSlot>,
    /// 진행 중 SUPER CLUTCH, 최대 5개, 회전 순서(첫 번째가 지금 히어로)
    pub active: Vec<HeroActive>,
}

const ACTIVE_LIMIT: usize = 5;

fn order(mut rows: Vec<Active>) -> Vec<Active> {
    rows.sort_by_key(|r| (r.shown_at.is_some(), r.shown_at, r.rotation_order, r.id));
    // 노출된 것 중 가장 늦은 것(맨 뒤)이 현재 슬롯이라 맨 앞으로.
    if rows.last().is_some_and(|r| r.shown_at.is_some()) {
        rows.rotate_right(1);
    }
    rows
}

/// 지금 슬롯을 시작해야 할 발동(현재 슬롯이 유효하면 `None`).
fn next_to_show(q: &[Active], slot: Duration) -> Option<i64> {
    let cur = q.first()?;
    match cur.shown_at {
        None => Some(cur.id),
        Some(s) if cur.now < s + slot => None,
        // 다음이 없으면(혼자) 같은 방송으로 새 슬롯
        Some(_) => Some(q.get(1).unwrap_or(cur).id),
    }
}

/// 회전을 한 걸음 진행한다(1초 작업·히어로 조회 공용). 잠금으로 직렬화해 동시에 불려도 한 번만 전환한다.
async fn advance(pool: &PgPool, slot: Duration) -> Result<Vec<Active>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    db::lock(&mut tx, "clutch:rotation").await?;
    let q = order(db::active(&mut *tx).await?);
    let Some(id) = next_to_show(&q, slot) else {
        return Ok(q);
    };
    db::mark_shown(&mut tx, id).await?;
    tx.commit().await?;
    Ok(order(db::active(pool).await?))
}

/// 1초 작업: 만료·종료된 발동을 빼고 30초가 지난 슬롯을 다음으로 넘긴다.
pub async fn rotate(state: Arc<AppState>) -> Result<(), AppError> {
    advance(
        &state.db,
        Duration::seconds(state.config.clutch.rotation_secs),
    )
    .await?;
    Ok(())
}

/// `GET /home/hero`(누구나). 1초 작업을 기다리지 않도록 슬롯이 끝났으면 여기서도 진행한다.
pub async fn hero(State(state): State<Arc<AppState>>) -> Result<Json<Hero>, AppError> {
    let cfg = &state.config.clutch;
    let slot = Duration::seconds(cfg.rotation_secs);
    let mut q = order(db::active(&state.db).await?);
    if next_to_show(&q, slot).is_some() {
        q = advance(&state.db, slot).await?;
    }
    let lives = broadcast::live_all(&state).await?;
    let active: Vec<(Active, Broadcast)> = q
        .into_iter()
        .filter_map(|r| {
            let b = lives.iter().find(|b| b.id == r.broadcast_id)?.clone();
            Some((r, b))
        })
        .collect();
    if let Some((cur, b)) = active.first() {
        return Ok(Json(Hero {
            mode: HeroMode::SuperClutch,
            gauge: Some(gauge_of(&state.db, cfg, b.id).await?),
            broadcast: Some(b.clone()),
            slot: cur.shown_at.map(|s| HeroSlot {
                started_at: s,
                next_switch_at: s + slot,
            }),
            active: active
                .iter()
                .take(ACTIVE_LIMIT)
                .map(|(r, b)| HeroActive {
                    broadcast: b.clone(),
                    expires_at: r.expires_at,
                })
                .collect(),
        }));
    }

    // 진행 중이 없으면 게이지 1위, 모두 0%면 시청자 수 1위(동률은 최신 방송).
    // shortcut: LIVE마다 게이지 쿼리 1번(N+1). LIVE가 수십 개를 넘으면 게이지를 한 쿼리로 모은다.
    let mut best: Option<(ClutchGauge, Broadcast)> = None;
    for b in lives {
        let g = gauge_of(&state.db, cfg, b.id).await?;
        let key = |g: &ClutchGauge, b: &Broadcast| (g.percent, b.viewer_count, b.id);
        if best
            .as_ref()
            .is_none_or(|(bg, bb)| key(&g, &b) > key(bg, bb))
        {
            best = Some((g, b));
        }
    }
    Ok(Json(match best {
        Some((g, b)) => Hero {
            mode: if g.percent > 0 {
                HeroMode::TopGauge
            } else {
                HeroMode::TopViewers
            },
            broadcast: Some(b),
            gauge: Some(g),
            slot: None,
            active: Vec::new(),
        },
        None => Hero {
            mode: HeroMode::Empty,
            broadcast: None,
            gauge: None,
            slot: None,
            active: Vec::new(),
        },
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: i64, order: i64, shown: Option<i64>, now: DateTime<Utc>) -> Active {
        Active {
            id,
            broadcast_id: id,
            rotation_order: order,
            shown_at: shown.map(|s| now - Duration::seconds(s)),
            expires_at: now + Duration::minutes(5),
            now,
        }
    }

    #[test]
    fn rotation_order_and_next() {
        let now = Utc::now();
        let slot = Duration::seconds(30);
        let ids = |q: &[Active]| q.iter().map(|r| r.id).collect::<Vec<_>>();
        // 현재(10초 전 노출) → 노출 전(order 동률은 id) → 오래전 노출
        let q = order(vec![
            row(1, 1, Some(70), now),
            row(2, 2, Some(10), now),
            row(4, 3, None, now),
            row(3, 3, None, now),
        ]);
        assert_eq!(ids(&q), [2, 3, 4, 1]);
        assert_eq!(next_to_show(&q, slot), None);
        // 슬롯이 끝나면 다음(노출 전 우선), 혼자면 자기 자신
        let q = order(vec![row(1, 1, Some(40), now), row(2, 2, None, now)]);
        assert_eq!(next_to_show(&q, slot), Some(2));
        let q = order(vec![row(1, 1, Some(31), now)]);
        assert_eq!(next_to_show(&q, slot), Some(1));
        assert_eq!(next_to_show(&[], slot), None);
    }
}
