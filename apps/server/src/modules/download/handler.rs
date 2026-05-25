use axum::{
    extract::{Path, Query, State},
    response::sse::{Event, Sse},
    Json,
};
use futures::stream::Stream;
use serde::Deserialize;
use std::convert::Infallible;
use std::sync::Arc;

use crate::app::AppState;
use crate::error::AppError;
use crate::models::download::{AddDownloadRequest, DownloadTask};
use crate::modules::download::service;

pub async fn add(
    State(state): State<Arc<AppState>>,
    _claims: crate::middleware::auth::AuthClaims,
    Json(req): Json<AddDownloadRequest>,
) -> Result<Json<DownloadTask>, AppError> {
    let task = service::add_task(&state.pool, req).await?;
    Ok(Json(task))
}

#[derive(Deserialize)]
pub struct ListQuery {
    pub status: Option<String>,
}

pub async fn list(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<DownloadTask>>, AppError> {
    let tasks = service::list_tasks(&state.pool, q.status.as_deref()).await?;
    Ok(Json(tasks))
}

pub async fn pause(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, AppError> {
    // If aria2 handles this task, pause via aria2 RPC
    if let Some(gid) = state.download_state.aria2.gid_map.lock().await.get(&id).cloned() {
        let _ = state.download_state.aria2.pause(&gid).await;
    } else {
        state.download_state.cancel_task(id).await;
    }
    service::pause_task(&state.pool, id).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn resume(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, AppError> {
    // If aria2 handles this task, resume via aria2 RPC
    if let Some(gid) = state.download_state.aria2.gid_map.lock().await.get(&id).cloned() {
        let _ = state.download_state.aria2.unpause(&gid).await;
    }
    service::resume_task(&state.pool, id).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn remove(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, AppError> {
    // If aria2 handles this task, remove via aria2 RPC
    if let Some(gid) = state.download_state.aria2.gid_map.lock().await.remove(&id) {
        let _ = state.download_state.aria2.remove(&gid).await;
    }
    service::remove_task(&state.pool, id).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn progress_sse(
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let pool = state.pool.clone();
    Sse::new(async_stream::stream! {
        loop {
            let tasks = service::list_tasks(&pool, None).await.unwrap_or_default();
            let data = serde_json::to_string(&tasks).unwrap_or_default();
            yield Ok(Event::default().data(data));
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    })
}
