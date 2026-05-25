use sqlx::SqlitePool;
use std::sync::Arc;

use crate::error::AppError;
use crate::models::file::{IndexedFile, Library, PaginatedResponse};
use crate::modules::library::scanner::{self, ScanState};

pub async fn create_library(
    pool: &SqlitePool,
    name: &str,
    path: &str,
    library_type: &str,
) -> Result<Library, AppError> {
    let now = chrono::Utc::now().timestamp_millis();
    let id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO libraries (name, path, library_type, created_at) VALUES (?, ?, ?, ?) RETURNING id",
    )
    .bind(name)
    .bind(path)
    .bind(library_type)
    .bind(now)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let library = sqlx::query_as::<_, Library>("SELECT * FROM libraries WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(library)
}

pub async fn list_libraries(pool: &SqlitePool) -> Result<Vec<Library>, AppError> {
    sqlx::query_as::<_, Library>("SELECT * FROM libraries ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn delete_library(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    let affected = sqlx::query("DELETE FROM libraries WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound("library not found".into()));
    }
    Ok(())
}

pub async fn trigger_scan(
    pool: &SqlitePool,
    library_id: i64,
    scan_state: Arc<ScanState>,
    thumb_dir: String,
) -> Result<(), AppError> {
    let library = sqlx::query_as::<_, Library>("SELECT * FROM libraries WHERE id = ?")
        .bind(library_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::NotFound("library not found".into()))?;

    let pool_clone = pool.clone();
    tokio::spawn(async move {
        if let Err(e) = scanner::scan_library(&pool_clone, &library, scan_state, &thumb_dir).await {
            tracing::error!("scan error: {}", e);
        }
    });

    Ok(())
}

pub fn get_scan_state(scan_state: &Arc<ScanState>) -> serde_json::Value {
    serde_json::json!({
        "status": *scan_state.status.lock(),
        "total": scan_state.total.load(std::sync::atomic::Ordering::SeqCst),
        "processed": scan_state.processed.load(std::sync::atomic::Ordering::SeqCst),
        "current_file": *scan_state.current_file.lock(),
    })
}

pub async fn list_movies(
    pool: &SqlitePool,
    cursor: Option<String>,
    page_size: i64,
) -> Result<PaginatedResponse<IndexedFile>, AppError> {
    let offset: i64 = cursor.and_then(|c| c.parse().ok()).unwrap_or(0);
    let items = sqlx::query_as::<_, IndexedFile>(
        "SELECT * FROM indexed_files WHERE file_type = 'video' ORDER BY title LIMIT ? OFFSET ?",
    )
    .bind(page_size + 1)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM indexed_files WHERE file_type = 'video'",
    )
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let has_more = items.len() > page_size as usize;
    let mut items = items;
    if has_more {
        items.truncate(items.len() - 1);
    }

    Ok(PaginatedResponse {
        next_cursor: if has_more {
            Some((offset + page_size).to_string())
        } else {
            None
        },
        total,
        items,
    })
}

pub async fn list_series(
    pool: &SqlitePool,
) -> Result<Vec<crate::models::series::Series>, AppError> {
    sqlx::query_as::<_, crate::models::series::Series>("SELECT * FROM series ORDER BY title")
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn get_media_detail(pool: &SqlitePool, id: i64) -> Result<IndexedFile, AppError> {
    sqlx::query_as::<_, IndexedFile>("SELECT * FROM indexed_files WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::NotFound("media not found".into()))
}

pub async fn search_media(pool: &SqlitePool, q: &str) -> Result<Vec<IndexedFile>, AppError> {
    let pattern = format!("%{}%", q);
    sqlx::query_as::<_, IndexedFile>(
        "SELECT * FROM indexed_files WHERE title LIKE ? AND file_type = 'video' LIMIT 50",
    )
    .bind(pattern)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))
}
