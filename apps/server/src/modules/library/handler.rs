use axum::{extract::{Path, Query, State}, Json};
use serde::Deserialize;
use std::sync::Arc;

use crate::app::AppState;
use crate::error::AppError;
use crate::modules::library::service;
use crate::models::file::{IndexedFile, PaginatedResponse};

pub async fn scan(
    State(state): State<Arc<AppState>>,
    _claims: crate::middleware::auth::AuthClaims,
) -> Result<Json<serde_json::Value>, AppError> {
    let libraries = sqlx::query_as::<_, crate::models::file::Library>("SELECT * FROM libraries")
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    for lib in &libraries {
        service::trigger_scan(&state.pool, lib.id, state.scan_state.clone()).await?;
    }

    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn scan_status(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    Json(service::get_scan_state(&state.scan_state))
}

#[derive(Deserialize)]
pub struct PageQuery {
    pub page: Option<i64>,
    pub size: Option<i64>,
}

pub async fn movies(
    State(state): State<Arc<AppState>>,
    Query(q): Query<PageQuery>,
) -> Result<Json<PaginatedResponse<IndexedFile>>, AppError> {
    let cursor = q.page.map(|p| ((p - 1) * q.size.unwrap_or(20)).to_string());
    let result = service::list_movies(&state.pool, cursor, q.size.unwrap_or(20)).await?;
    Ok(Json(result))
}

pub async fn series_list(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<crate::models::series::Series>>, AppError> {
    let series = service::list_series(&state.pool).await?;
    Ok(Json(series))
}

pub async fn media_detail(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<IndexedFile>, AppError> {
    let media = service::get_media_detail(&state.pool, id).await?;
    Ok(Json(media))
}

pub async fn refresh(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, AppError> {
    let now = chrono::Utc::now().timestamp_millis();
    sqlx::query("UPDATE indexed_files SET status = 'pending', updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(serde_json::json!({"ok": true})))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

pub async fn search(
    State(state): State<Arc<AppState>>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<Vec<IndexedFile>>, AppError> {
    let results = service::search_media(&state.pool, &q.q).await?;
    Ok(Json(results))
}
