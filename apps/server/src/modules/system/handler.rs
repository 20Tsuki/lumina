use axum::{extract::State, Json};
use std::sync::Arc;

use crate::app::AppState;
use crate::error::AppError;
use crate::modules::system::service;

pub async fn info() -> Json<serde_json::Value> {
    Json(service::get_system_info())
}

pub async fn get_settings(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let settings = service::get_settings(&state.pool).await?;
    Ok(Json(settings))
}

pub async fn update_settings(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    service::update_settings(&state.pool, body).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}
