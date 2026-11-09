//! `/api/v1` 라우터 조립. 모든 도메인이 미리 연결돼 있다 — 도메인은 자기 `mod.rs`의 `routes()`만 채운다.

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use ts_rs::TS;

use crate::AppState;

pub fn api() -> Router<Arc<AppState>> {
    Router::new()
        .route("/health", get(health))
        .merge(crate::auth::routes())
        .merge(crate::broadcast::routes())
        .merge(crate::live::routes())
        .merge(crate::chat::routes())
        .merge(crate::moderation::routes())
        .merge(crate::wallet::routes())
        .merge(crate::clutch::routes())
        .merge(crate::recording::routes())
        .merge(crate::follow::routes())
        .merge(crate::realtime::routes())
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct Health {
    /// `ok` 또는 `unavailable`(DB 연결 불가)
    pub status: String,
}

async fn health(State(state): State<Arc<AppState>>) -> (StatusCode, Json<Health>) {
    match sqlx::query_scalar!(r#"SELECT 1 AS "one!""#)
        .fetch_one(&state.db)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(Health {
                status: "ok".into(),
            }),
        ),
        Err(e) => {
            tracing::error!(error = %e, "health: database unavailable");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(Health {
                    status: "unavailable".into(),
                }),
            )
        }
    }
}
