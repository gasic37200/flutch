//! 녹화·다시보기 도메인 — docs/features/08-recording.md
//! Egress 시작은 송출자가 방에 들어온 뒤(방이 있어야 시작된다), 종료는 `broadcast::end_live`,
//! 완료는 `egress_ended` 웹훅. 녹화 상태는 방송 상태와 분리돼 있어 어떤 실패도 방송을 막지 않는다.

use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::routing::{get, post};
use chrono::DateTime;
use serde_json::Value;

use crate::AppState;
use crate::broadcast::{RecordingRef, RecordingStatus, room_name};
use crate::error::AppError;
use crate::realtime::{Event, WsEvent};
use crate::scheduler::Scheduler;

mod db;
mod handlers;
mod storage;

pub use handlers::{RecordingCard, RecordingDetail};
pub use storage::{NoStorage, ObjectStore, S3Http, StorageError};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/recordings", get(handlers::list))
        .route("/recordings/{id}", get(handlers::get_one))
        .route("/recordings/{id}/views", post(handlers::add_view))
}

/// 주기 작업: 보관 정리(1시간)
pub fn register(s: &mut Scheduler) {
    s.every(
        "recording_cleanup",
        Duration::from_secs(3600),
        |state| async move { cleanup(&state).await },
    );
}

/// 저장소 객체 키. 방송당 녹화 1개.
fn object_key(broadcast_id: i64) -> String {
    format!("recordings/{broadcast_id}.mp4")
}

fn to_ref(r: db::Ref) -> Result<RecordingRef, AppError> {
    Ok(RecordingRef {
        id: r.id,
        status: parse_status(&r.status)?,
    })
}

fn parse_status(s: &str) -> Result<RecordingStatus, AppError> {
    serde_json::from_value(Value::String(s.to_owned()))
        .map_err(|_| AppError::Internal(format!("unexpected recording status {s:?}")))
}

fn publish_updated(state: &AppState, broadcast_id: i64, id: i64, status: RecordingStatus) {
    state.realtime.publish(
        broadcast_id,
        Event::public(WsEvent::RecordingUpdated(RecordingRef { id, status })),
    );
}

/// 방송의 녹화 `{id, status}`(없으면 `None`). `broadcast.ended`가 싣는다.
pub async fn ref_of(state: &AppState, broadcast_id: i64) -> Result<Option<RecordingRef>, AppError> {
    db::ref_of(&state.db, broadcast_id)
        .await?
        .map(to_ref)
        .transpose()
}

/// 송출자가 방에 들어왔다 → 녹화 켜짐이면 Egress 시작. 같은 방송에서 여러 번 불려도 한 번만 시작한다.
/// 시작 실패는 녹화만 `failed`로 하고 방송은 건드리지 않는다. 자동 재시도 없음(features/08).
/// shortcut: 송출자가 끊겼다 돌아와 방이 새로 만들어지면 Egress는 이어지지 않는다(앞 구간만 저장), 재시도 정책이 생길 때.
pub async fn start(state: &AppState, broadcast_id: i64) -> Result<(), AppError> {
    let Some(s3) = &state.config.recording.s3 else {
        return Ok(());
    };
    let key = object_key(broadcast_id);
    let Some(id) = db::insert_for_live(&state.db, broadcast_id, &key).await? else {
        return Ok(());
    };
    match state
        .livekit
        .start_recording(&room_name(broadcast_id), s3, &key)
        .await
    {
        Ok(egress_id) => {
            db::set_egress(&state.db, id, &egress_id).await?;
            tracing::info!(broadcast_id, %egress_id, "recording started");
        }
        Err(e) => {
            tracing::error!(broadcast_id, error = %e, "recording start failed");
            db::fail_start(&state.db, id).await?;
            publish_updated(state, broadcast_id, id, RecordingStatus::Failed);
        }
    }
    Ok(())
}

/// 방송 종료 시 Egress를 멈춰 파일을 마무리하게 한다. 실패해도 종료는 유지(로그만) — 방을 닫으면 Egress도 끝난다.
pub async fn stop(state: &AppState, broadcast_id: i64) {
    let egress = match db::running_egress(&state.db, broadcast_id).await {
        Ok(e) => e,
        Err(e) => {
            tracing::error!(broadcast_id, error = %e, "recording lookup failed");
            return;
        }
    };
    if let Some(egress_id) = egress
        && let Err(e) = state.livekit.stop_recording(&egress_id).await
    {
        tracing::error!(broadcast_id, %egress_id, error = %e, "recording stop failed");
    }
}

/// `egress_ended` 웹훅(`egressInfo`, protojson). int64는 문자열로 온다. 반복돼도 한 번만 반영(멱등).
pub async fn egress_ended(state: &AppState, info: &Value) -> Result<(), AppError> {
    let Some(egress_id) = info["egressId"].as_str().or(info["egress_id"].as_str()) else {
        return Ok(());
    };
    let file = info["fileResults"]
        .get(0)
        .or_else(|| info.get("file"))
        .filter(|f| f.is_object());
    let num = |v: &Value| v.as_i64().or_else(|| v.as_str()?.parse().ok());
    let size = file.and_then(|f| num(&f["size"])).filter(|n| *n > 0);
    let duration_ns = file.and_then(|f| num(&f["duration"])).unwrap_or(0);
    // 영상 0초의 실제 시각(나노초): 파일 시작, 없으면 Egress 시작. 송출 시작보다 몇 초 늦어 다시보기 동기화 기준이 된다.
    let video_started_at = file
        .and_then(|f| num(&f["startedAt"]))
        .or_else(|| num(&info["startedAt"]))
        .filter(|n| *n > 0)
        .map(DateTime::from_timestamp_nanos);
    let done = info["status"] == "EGRESS_COMPLETE";

    let (finished, status) = match size {
        Some(size) if done => (
            db::finish_ready(
                &state.db,
                egress_id,
                size,
                ((duration_ns + 500_000_000) / 1_000_000_000) as i32,
                state.config.recording.retention_days,
                video_started_at,
            )
            .await?,
            RecordingStatus::Ready,
        ),
        _ => {
            tracing::warn!(%egress_id, status = %info["status"], error = %info["error"], "recording failed");
            (
                db::finish_failed(&state.db, egress_id).await?,
                RecordingStatus::Failed,
            )
        }
    };
    match finished {
        Some(f) => publish_updated(state, f.broadcast_id, f.id, status),
        // 이미 처리됐거나, 끝나기 전에 방송을 지워 deleted가 된 녹화 → 방금 올라온 파일을 지운다.
        // 삭제 때의 즉시 정리는 업로드 전이라 빈 키를 지우고 표시(키)를 비웠을 수 있어 다시 삭제 대기로 둔다.
        None => {
            if let Some((id, broadcast_id)) = db::deleted_without_key(&state.db, egress_id).await? {
                db::requeue_delete(&state.db, id, &object_key(broadcast_id)).await?;
            }
            purge(state).await?
        }
    }
    Ok(())
}

/// deleted인데 파일이 남은 녹화의 객체를 지운다. 실패한 것은 다음 호출(1시간 주기)이 다시 시도한다.
pub async fn purge(state: &AppState) -> Result<(), AppError> {
    for (id, key) in db::pending_deletes(&state.db).await? {
        match state.storage.delete(&key).await {
            Ok(()) => db::clear_key(&state.db, id).await?,
            Err(e) => {
                tracing::error!(recording_id = id, error = %e, "recording object delete failed")
            }
        }
    }
    Ok(())
}

/// 보관 정리: 30일 지난 것 → 16GB 초과분(오래된 것부터) → 종료 통보가 없는 녹화 실패 처리 → 객체 삭제.
pub async fn cleanup(state: &AppState) -> Result<(), AppError> {
    let c = &state.config.recording;
    let expired = db::expire_old(&state.db).await?;
    let over = db::expire_over_capacity(&state.db, c.max_total_bytes).await?;
    // 방송 최대 길이 + 1시간 안에 끝나지 않은 녹화는 웹훅을 놓친 것으로 본다
    let stale_secs = (state.config.live.max_duration_secs + 3600) as f64;
    let stale = db::fail_stale(&state.db, stale_secs).await?;
    if expired + over + stale > 0 {
        tracing::info!(expired, over, stale, "recording cleanup");
    }
    purge(state).await
}
