use std::fs;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod app;
mod config;
mod db;
mod error;
mod middleware;
mod models;
mod modules;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::load()?;
    let pool = db::init(&config).await?;
    fs::create_dir_all(config.thumbnail_dir())?;

    let user_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await?;

    if user_count.0 == 0 {
        modules::auth::service::create_user(&pool, "admin", "admin123", "admin").await?;
        tracing::info!("created default admin user (username: admin, password: admin123)");
    }

    let scan_state = modules::library::scanner::ScanState::new();

    // Create download directory and start BT session via librqbit (built-in, no external deps)
    let _ = std::fs::create_dir_all(config.download_dir());
    let bt = modules::download::bittorrent::BtManager::new(
        config.download_dir().to_string_lossy().as_ref(),
    )
    .await
    .expect("failed to init librqbit BT session");

    let download_state = modules::download::service::DownloadState::new(bt);
    let state = app::AppState::new(config, pool, scan_state, download_state.clone());

    // Spawn download worker
    let worker_pool = state.pool.clone();
    let worker_state = download_state.clone();
    tokio::spawn(async move {
        modules::download::client::run_worker(worker_pool, worker_state).await;
    });

    let listener = tokio::net::TcpListener::bind(state.config.server_addr()).await?;
    tracing::info!("listening on {}", state.config.server_addr());

    axum::serve(listener, app::build(state)).await?;
    Ok(())
}
