use std::error::Error;
use std::sync::Arc;

use flutch::config::Config;
use flutch::livekit::LiveKitHttp;
use flutch::scheduler::Scheduler;
use flutch::toss::TossHttp;
use flutch::{AppState, app, connect_db, realtime};
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

    let http = reqwest::Client::new();
    let port = config.backend_port;
    let state = Arc::new(AppState {
        livekit: Arc::new(LiveKitHttp::new(config.livekit.clone(), http.clone())),
        toss: Arc::new(TossHttp::new(http)),
        realtime: realtime::Hub::default(),
        db,
        config,
    });

    Scheduler::with_all_domains().start(state.clone());

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    tracing::info!(port, "listening");
    axum::serve(listener, app(state)).await?;
    Ok(())
}
