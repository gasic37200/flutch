use std::error::Error;
use std::sync::Arc;

use flutch::config::Config;
use flutch::livekit::LiveKitHttp;
use flutch::scheduler::Scheduler;
use flutch::toss::TossHttp;
use flutch::{AppState, app, connect_db, realtime, recording};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::time::ChronoLocal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // .env는 로컬 개발용. 운영은 컨테이너 환경변수만 쓰므로 파일이 없어도 된다.
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .json()
        .with_timer(ChronoLocal::rfc_3339())
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let config = Config::from_env()?;
    tracing::info!(?config, "config loaded");

    let db = connect_db(&config.database_url).await?;
    sqlx::migrate!().run(&db).await?;

    // 외부(LiveKit·토스) 응답이 멈춰도 스케줄러 작업이 무한정 붙잡히지 않게 한다.
    let http = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(5))
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    let port = config.backend_port;
    let storage: Arc<dyn recording::ObjectStore> = match &config.recording.s3 {
        Some(s3) => Arc::new(recording::S3Http::new(s3.clone(), http.clone())),
        None => Arc::new(recording::NoStorage),
    };
    let state = Arc::new(AppState {
        livekit: Arc::new(LiveKitHttp::new(config.livekit.clone(), http.clone())),
        toss: Arc::new(TossHttp::new(http)),
        storage,
        realtime: realtime::Hub::default(),
        live: Default::default(),
        clutch: Default::default(),
        db,
        config,
    });

    Scheduler::with_all_domains().start(state.clone());

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    tracing::info!(port, "listening");
    axum::serve(listener, app(state))
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

// docker stop(SIGTERM)과 로컬 Ctrl+C 모두에서 진행 중 요청을 마무리하고 종료한다.
async fn shutdown_signal() {
    use tokio::signal::unix::{SignalKind, signal};
    let mut term = signal(SignalKind::terminate()).expect("SIGTERM handler");
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = term.recv() => {}
    }
    tracing::info!("shutdown signal received");
}
