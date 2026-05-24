use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::error::AppError;
use crate::models::download::{AddDownloadRequest, DownloadTask};

pub struct DownloadState {
    pub tx: broadcast::Sender<Vec<DownloadTask>>,
}

impl DownloadState {
    pub fn new() -> Arc<Self> {
        let (tx, _) = broadcast::channel(100);
        Arc::new(Self { tx })
    }
}

pub async fn add_task(
    pool: &SqlitePool,
    req: AddDownloadRequest,
) -> Result<DownloadTask, AppError> {
    let now = chrono::Utc::now().timestamp_millis();
    let id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO download_tasks (url, save_path, status, created_at, updated_at) VALUES (?, ?, 'queued', ?, ?) RETURNING id",
    )
    .bind(&req.url)
    .bind(&req.save_path)
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let task = sqlx::query_as::<_, DownloadTask>("SELECT * FROM download_tasks WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(task)
}

pub async fn list_tasks(
    pool: &SqlitePool,
    status: Option<&str>,
) -> Result<Vec<DownloadTask>, AppError> {
    let tasks = if let Some(s) = status {
        sqlx::query_as::<_, DownloadTask>(
            "SELECT * FROM download_tasks WHERE status = ? ORDER BY created_at DESC",
        )
        .bind(s)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, DownloadTask>(
            "SELECT * FROM download_tasks ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await
    };
    tasks.map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn pause_task(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    let now = chrono::Utc::now().timestamp_millis();
    sqlx::query(
        "UPDATE download_tasks SET status = 'paused', updated_at = ? WHERE id = ? AND status IN ('queued', 'downloading')",
    )
    .bind(now)
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

pub async fn resume_task(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    let now = chrono::Utc::now().timestamp_millis();
    sqlx::query(
        "UPDATE download_tasks SET status = 'queued', updated_at = ? WHERE id = ? AND status = 'paused'",
    )
    .bind(now)
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

pub async fn remove_task(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    sqlx::query("DELETE FROM download_tasks WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}
