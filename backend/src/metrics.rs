//! Prometheus `/metrics`. nginx가 외부 요청은 404로 막고, Prometheus가 도커 네트워크로 직접 수집한다.

use std::sync::{Arc, OnceLock};

use axum::extract::State;
use axum::response::IntoResponse;
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};

use crate::AppState;

static HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();

/// 전역 recorder를 한 번만 설치한다(시나리오 테스트가 앱을 여러 번 만들어도 안전).
pub fn handle() -> &'static PrometheusHandle {
    HANDLE.get_or_init(|| {
        let h = PrometheusBuilder::new()
            // 지연은 요약(summary)이 아니라 히스토그램으로 내보내야 PromQL로 p95를 낼 수 있다.
            .set_buckets_for_metric(
                Matcher::Full("http_request_duration_seconds".into()),
                &[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0],
            )
            .expect("valid buckets")
            .install_recorder()
            .expect("metrics recorder");
        // 후원 카운터 자리: 값이 0이어도 시리즈가 보이도록 먼저 만든다(후원 구현이 increment).
        metrics::counter!("donations_total").increment(0);
        h
    })
}

/// 요청 하나를 기록한다. `route`는 경로 템플릿(예: `/api/v1/broadcasts/{id}`)이라 id가 라벨에 들어가지 않는다.
pub fn record_request(method: &str, route: &str, status: u16, secs: f64) {
    metrics::counter!(
        "http_requests_total",
        "method" => method.to_owned(),
        "route" => route.to_owned(),
        "status" => status.to_string()
    )
    .increment(1);
    metrics::histogram!(
        "http_request_duration_seconds",
        "method" => method.to_owned(),
        "route" => route.to_owned()
    )
    .record(secs);
}

pub async fn scrape(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // 게이지는 스크레이프 때 계산한다. DB 조회가 실패하면 LIVE 수만 이전 값을 유지한다.
    match sqlx::query_scalar!(r#"SELECT count(*) AS "n!" FROM broadcasts WHERE status = 'LIVE'"#)
        .fetch_one(&state.db)
        .await
    {
        Ok(n) => metrics::gauge!("live_broadcasts").set(n as f64),
        Err(e) => tracing::error!(error = %e, "metrics: live count failed"),
    }
    metrics::gauge!("ws_connections").set(state.realtime.connections() as f64);
    (
        [("content-type", "text/plain; version=0.0.4")],
        handle().render(),
    )
}
