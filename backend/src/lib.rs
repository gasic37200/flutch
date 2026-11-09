pub mod config;
pub mod error;
pub mod extract;
pub mod livekit;
pub mod middleware;
pub mod realtime;
pub mod routes;
pub mod scheduler;
pub mod toss;

pub mod auth;
pub mod broadcast;
pub mod chat;
pub mod clutch;
pub mod follow;
pub mod live;
pub mod moderation;
pub mod recording;
pub mod wallet;

use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use config::Config;
use livekit::LiveKitApi;
use toss::TossApi;

/// 모든 핸들러·주기 작업이 공유하는 상태. Axum `State<Arc<AppState>>`로 받는다.
pub struct AppState {
    pub config: Config,
    pub db: PgPool,
    pub livekit: Arc<dyn LiveKitApi>,
    pub toss: Arc<dyn TossApi>,
    pub realtime: realtime::Hub,
    /// 송출 메모리 상태(시청자 집합 등)
    pub live: live::LiveState,
}

/// 전체 앱 라우터. `main`과 시나리오 테스트가 같은 조립을 쓴다.
pub fn app(state: Arc<AppState>) -> Router {
    Router::new()
        .nest("/api/v1", routes::api())
        .fallback(|| async { error::AppError::NotFound })
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::check_origin,
        ))
        .layer(axum::middleware::from_fn(middleware::request_context))
        .with_state(state)
}

/// DB 풀. 연결마다 UTC로 고정한다(architecture §7-1).
pub async fn connect_db(url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                sqlx::query("SET TIME ZONE 'UTC'").execute(conn).await?;
                Ok(())
            })
        })
        .connect(url)
        .await
}
