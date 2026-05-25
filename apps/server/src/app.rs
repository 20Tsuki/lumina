use axum::{http::Method, Router};
use sqlx::SqlitePool;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::config::Config;
use crate::modules::library::scanner::ScanState;
use crate::modules::download::service::DownloadState;

pub struct AppState {
    pub config: Config,
    pub pool: SqlitePool,
    pub scan_state: Arc<ScanState>,
    pub download_state: Arc<DownloadState>,
}

impl AppState {
    pub fn new(config: Config, pool: SqlitePool, scan_state: Arc<ScanState>, download_state: Arc<DownloadState>) -> Arc<Self> {
        Arc::new(Self { config, pool, scan_state, download_state })
    }
}

pub fn build(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any);

    Router::new()
        .route("/health", axum::routing::get(|| async { "ok" }))
        .nest("/api/auth", auth_routes())
        .nest("/api/files", files_routes())
        .nest("/api/library", library_routes())
        .nest("/api/media", media_routes())
        .nest("/api/stream", stream_routes())
        .nest("/api/download", download_routes())
        .nest("/api/system", system_routes())
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", axum::routing::post(crate::modules::auth::handler::login))
        .route("/logout", axum::routing::post(crate::modules::auth::handler::logout))
        .route("/me", axum::routing::get(crate::modules::auth::handler::me))
}

fn files_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/list", axum::routing::get(crate::modules::files::handler::list))
        .route("/mkdir", axum::routing::post(crate::modules::files::handler::mkdir))
        .route("/move", axum::routing::post(crate::modules::files::handler::move_file))
        .route("/delete", axum::routing::post(crate::modules::files::handler::delete))
        .route("/upload", axum::routing::post(crate::modules::files::handler::upload))
        .route("/download", axum::routing::get(crate::modules::files::handler::download))
        .route("/thumbnail", axum::routing::get(crate::modules::files::handler::thumbnail))
}

fn library_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", axum::routing::get(crate::modules::library::handler::list_libraries))
        .route("/", axum::routing::post(crate::modules::library::handler::create_library))
        .route("/{id}", axum::routing::delete(crate::modules::library::handler::delete_library))
        .route("/scan", axum::routing::post(crate::modules::library::handler::scan))
        .route("/status", axum::routing::get(crate::modules::library::handler::scan_status))
}

fn media_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/movies", axum::routing::get(crate::modules::library::handler::movies))
        .route("/series", axum::routing::get(crate::modules::library::handler::series_list))
        .route("/search", axum::routing::get(crate::modules::library::handler::search))
        .route("/{id}", axum::routing::get(crate::modules::library::handler::media_detail))
        .route("/{id}/refresh", axum::routing::post(crate::modules::library::handler::refresh))
}

fn stream_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/{id}/file", axum::routing::get(crate::modules::stream::handler::serve_file))
}

fn download_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/add", axum::routing::post(crate::modules::download::handler::add))
        .route("/list", axum::routing::get(crate::modules::download::handler::list))
        .route("/{id}/pause", axum::routing::post(crate::modules::download::handler::pause))
        .route("/{id}/resume", axum::routing::post(crate::modules::download::handler::resume))
        .route("/{id}/remove", axum::routing::post(crate::modules::download::handler::remove))
        .route("/progress", axum::routing::get(crate::modules::download::handler::progress_sse))
}

fn system_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/info", axum::routing::get(crate::modules::system::handler::info))
        .route("/settings", axum::routing::get(crate::modules::system::handler::get_settings))
        .route("/settings", axum::routing::put(crate::modules::system::handler::update_settings))
}
