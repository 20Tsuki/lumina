use std::fs;
use std::path::Path;
use std::process::Command;
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

pub async fn scan_library(pool: &SqlitePool, library: &Library, state: Arc<ScanState>, thumb_dir: &str) -> Result<(), AppError> {
    if state.running.swap(true, Ordering::SeqCst) {
        return Err(AppError::Conflict("scan already running".into()));
    }

    *state.status.lock() = "scanning".into();
    state.total.store(0, Ordering::SeqCst);
    state.processed.store(0, Ordering::SeqCst);

    let result = do_scan(pool, library, &state, thumb_dir).await;

    if result.is_err() {
        *state.status.lock() = "error".into();
    } else {
        *state.status.lock() = "done".into();
    }
    state.running.store(false, Ordering::SeqCst);
    result
}

async fn do_scan(pool: &SqlitePool, library: &Library, state: &Arc<ScanState>, thumb_dir: &str) -> Result<(), AppError> {
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

        // Extract metadata for video files
        if file_type == "video" {
            let full_path = Path::new(&library.path).join(relative_path);
            let file_id: i64 = sqlx::query_scalar(
                "SELECT id FROM indexed_files WHERE library_id = ? AND file_path = ?"
            )
            .bind(library.id)
            .bind(relative_path)
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

            if let Some(meta) = extract_metadata(&full_path, thumb_dir) {
                let thumb_path = if let Some(ref thumb) = meta.thumb_rel {
                    Some(thumb.clone())
                } else {
                    None
                };
                let _ = sqlx::query(
                    "UPDATE indexed_files SET codec = ?, resolution = ?, duration = ?, bitrate = ?, thumb_path = ?, metadata_json = ? WHERE id = ?"
                )
                .bind(&meta.codec)
                .bind(&meta.resolution)
                .bind(meta.duration)
                .bind(meta.bitrate)
                .bind(&thumb_path)
                .bind(&meta.metadata_json)
                .bind(file_id)
                .execute(pool)
                .await;
            }
        }
    }

    Ok(())
}

struct VideoMeta {
    codec: Option<String>,
    resolution: Option<String>,
    duration: Option<i64>,
    bitrate: Option<i64>,
    metadata_json: Option<String>,
    thumb_rel: Option<String>,
}

fn extract_metadata(file_path: &Path, thumb_dir: &str) -> Option<VideoMeta> {
    let output = Command::new("ffprobe")
        .args([
            "-v", "quiet",
            "-print_format", "json",
            "-show_format",
            "-show_streams",
        ])
        .arg(file_path)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&json_str).ok()?;

    let streams = parsed["streams"].as_array()?;
    let video_stream = streams.iter().find(|s| s["codec_type"].as_str() == Some("video"));

    let codec = video_stream.and_then(|s| s["codec_name"].as_str()).map(String::from);
    let resolution = video_stream.and_then(|s| {
        let w = s["width"].as_i64()?;
        let h = s["height"].as_i64()?;
        Some(format!("{}x{}", w, h))
    });

    let duration = parsed["format"]["duration"]
        .as_str()
        .and_then(|d| d.parse::<f64>().ok())
        .map(|d| d as i64);

    let bitrate = parsed["format"]["bit_rate"]
        .as_str()
        .and_then(|b| b.parse::<i64>().ok());

    // Generate thumbnail
    let thumb_rel = generate_thumbnail(file_path, thumb_dir);

    Some(VideoMeta {
        codec,
        resolution,
        duration,
        bitrate,
        metadata_json: Some(json_str.to_string()),
        thumb_rel,
    })
}

fn generate_thumbnail(file_path: &Path, thumb_dir: &str) -> Option<String> {
    // Use MD5 of path as thumbnail filename
    let hash = format!("{:x}", md5::compute(file_path.to_string_lossy().as_bytes()));
    let thumb_name = format!("{}.jpg", hash);

    let thumb_dir_path = Path::new(thumb_dir);
    let thumb_path = thumb_dir_path.join(&thumb_name);

    if thumb_path.exists() {
        return Some(thumb_name);
    }

    fs::create_dir_all(&thumb_dir).ok()?;

    let status = Command::new("ffmpeg")
        .args([
            "-ss", "10",
            "-i",
        ])
        .arg(file_path)
        .args([
            "-vframes", "1",
            "-vf", "scale=320:180",
            "-y",
        ])
        .arg(&thumb_path)
        .output()
        .ok()
        .map(|o| o.status.success())?;

    if status {
        Some(thumb_name)
    } else {
        None
    }
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
