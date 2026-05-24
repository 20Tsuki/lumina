use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::Arc;

use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::file::Library;

pub struct ScanState {
    pub status: parking_lot::Mutex<String>,
    pub total: AtomicI64,
    pub processed: AtomicI64,
    pub current_file: parking_lot::Mutex<String>,
    pub running: AtomicBool,
}

impl ScanState {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            status: parking_lot::Mutex::new("idle".into()),
            total: AtomicI64::new(0),
            processed: AtomicI64::new(0),
            current_file: parking_lot::Mutex::new(String::new()),
            running: AtomicBool::new(false),
        })
    }
}

pub async fn scan_library(pool: &SqlitePool, library: &Library, state: Arc<ScanState>) -> Result<(), AppError> {
    if state.running.swap(true, Ordering::SeqCst) {
        return Err(AppError::Conflict("scan already running".into()));
    }

    *state.status.lock() = "scanning".into();
    state.total.store(0, Ordering::SeqCst);
    state.processed.store(0, Ordering::SeqCst);

    let result = do_scan(pool, library, &state).await;

    if result.is_err() {
        *state.status.lock() = "error".into();
    } else {
        *state.status.lock() = "done".into();
    }
    state.running.store(false, Ordering::SeqCst);
    result
}

async fn do_scan(pool: &SqlitePool, library: &Library, state: &Arc<ScanState>) -> Result<(), AppError> {
    let mut files = Vec::new();
    walk_dir(Path::new(&library.path), Path::new(&library.path), &mut files)?;

    state.total.store(files.len() as i64, Ordering::SeqCst);

    let now = chrono::Utc::now().timestamp_millis();
    for (i, (relative_path, size)) in files.iter().enumerate() {
        state.processed.store(i as i64 + 1, Ordering::SeqCst);
        *state.current_file.lock() = relative_path.clone();

        let file_type = classify_by_extension(relative_path);
        let title = Path::new(relative_path)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        sqlx::query(
            "INSERT INTO indexed_files (library_id, file_path, file_type, title, size, status, created_at, updated_at) VALUES (?, ?, ?, ?, ?, 'pending', ?, ?) ON CONFLICT(library_id, file_path) DO UPDATE SET size=?, updated_at=?"
        )
        .bind(library.id)
        .bind(relative_path)
        .bind(&file_type)
        .bind(&title)
        .bind(*size)
        .bind(now)
        .bind(now)
        .bind(*size)
        .bind(now)
        .execute(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    }

    Ok(())
}

fn walk_dir(root: &Path, dir: &Path, files: &mut Vec<(String, i64)>) -> Result<(), AppError> {
    let entries = fs::read_dir(dir).map_err(|e| AppError::Internal(format!("read_dir: {}", e)))?;
    for entry in entries {
        let entry = entry.map_err(|e| AppError::Internal(e.to_string()))?;
        let metadata = entry.metadata().map_err(|e| AppError::Internal(e.to_string()))?;
        let name = entry.file_name().to_string_lossy().to_string();

        if name.starts_with('.') { continue; }

        if metadata.is_dir() {
            walk_dir(root, &entry.path(), files)?;
        } else {
            let entry_path = entry.path();
            let relative = entry_path.strip_prefix(root).unwrap_or(&entry_path);
            files.push((relative.to_string_lossy().to_string(), metadata.len() as i64));
        }
    }
    Ok(())
}

fn classify_by_extension(path: &str) -> String {
    let ext = Path::new(path).extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    if ["mp4", "mkv", "avi", "mov", "m2ts", "ts", "wmv", "flv"].contains(&ext.as_str()) { return "video".into(); }
    if ["jpg", "jpeg", "png", "gif", "webp", "heic", "bmp"].contains(&ext.as_str()) { return "image".into(); }
    if ["mp3", "flac", "aac", "wav", "ogg"].contains(&ext.as_str()) { return "audio".into(); }
    if ["zip", "rar", "7z", "tar", "gz", "bz2"].contains(&ext.as_str()) { return "archive".into(); }
    if ["pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "txt", "md"].contains(&ext.as_str()) { return "document".into(); }
    "other".into()
}
