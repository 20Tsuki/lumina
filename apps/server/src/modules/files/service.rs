use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use image::GenericImageView;

use crate::error::AppError;
use crate::models::file::FileEntry;

pub fn list_files(root: &Path, relative_path: &str) -> Result<Vec<FileEntry>, AppError> {
    let dir = resolve_path(root, relative_path)?;
    if !dir.is_dir() {
        return Err(AppError::NotFound("not a directory".into()));
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e| AppError::Internal(format!("read_dir: {}", e)))? {
        let entry = entry.map_err(|e| AppError::Internal(e.to_string()))?;
        let metadata = entry.metadata().map_err(|e| AppError::Internal(e.to_string()))?;
        let name = entry.file_name().to_string_lossy().to_string();

        if name.starts_with('.') {
            continue;
        }

        let modified_at = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);

        let is_dir = metadata.is_dir();
        let size = if is_dir { 0 } else { metadata.len() as i64 };
        let file_type = classify_file(&name, is_dir);
        let thumbnail_url = if file_type == "image" {
            Some(format!("/api/files/thumbnail?path={}&size=256", urlencoding(&compound_path(relative_path, &name))))
        } else {
            None
        };

        entries.push(FileEntry {
            name,
            path: compound_path(relative_path, &entry.file_name().to_string_lossy()),
            file_type,
            size,
            is_dir,
            modified_at,
            thumbnail_url,
        });
    }

    entries.sort_by(|a, b| {
        if a.is_dir != b.is_dir { return if a.is_dir { std::cmp::Ordering::Less } else { std::cmp::Ordering::Greater }; }
        a.name.to_lowercase().cmp(&b.name.to_lowercase())
    });

    Ok(entries)
}

pub fn mkdir(root: &Path, relative_path: &str, name: &str) -> Result<(), AppError> {
    let parent = resolve_path(root, relative_path)?;
    fs::create_dir_all(parent.join(name)).map_err(|e| AppError::Internal(e.to_string()))
}

pub fn move_file(root: &Path, from: &str, to: &str) -> Result<(), AppError> {
    let src = resolve_path(root, from)?;
    let dst = resolve_path(root, to)?;
    fs::rename(&src, &dst).map_err(|e| AppError::Internal(e.to_string()))
}

pub fn delete(root: &Path, relative_path: &str) -> Result<(), AppError> {
    let trash = root.join(".trash");
    fs::create_dir_all(&trash).ok();
    let src = resolve_path(root, relative_path)?;
    let name = src.file_name().unwrap_or_default().to_string_lossy();
    let dst = trash.join(format!("{}_{}", chrono::Utc::now().timestamp_millis(), name));
    fs::rename(&src, &dst).map_err(|e| AppError::Internal(e.to_string()))
}

pub fn download_path(root: &Path, relative_path: &str) -> Result<PathBuf, AppError> {
    let path = resolve_path(root, relative_path)?;
    if !path.is_file() {
        return Err(AppError::NotFound("not a file".into()));
    }
    Ok(path)
}

pub fn generate_thumbnail(root: &Path, relative_path: &str, size: u32, thumb_dir: &Path) -> Result<PathBuf, AppError> {
    let src = resolve_path(root, relative_path)?;
    let hash = format!("{:x}", md5::compute(relative_path));
    let thumb_path = thumb_dir.join(format!("{}_{}.jpg", hash, size));

    if thumb_path.exists() {
        return Ok(thumb_path);
    }

    fs::create_dir_all(thumb_dir).ok();

    let img = image::open(&src).map_err(|e| AppError::Internal(format!("image open: {}", e)))?;
    let thumb = img.thumbnail(size, size);
    thumb.save(&thumb_path).map_err(|e| AppError::Internal(format!("save thumbnail: {}", e)))?;

    Ok(thumb_path)
}

fn resolve_path(root: &Path, relative_path: &str) -> Result<PathBuf, AppError> {
    let sanitized = relative_path.trim_start_matches('/');
    let resolved = root.join(sanitized);
    let canonical_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let canonical = resolved.canonicalize().unwrap_or_else(|_| resolved.clone());
    if !canonical.starts_with(&canonical_root) && resolved != *root {
        return Err(AppError::BadRequest("path traversal denied".into()));
    }
    Ok(resolved)
}

fn compound_path(base: &str, name: &str) -> String {
    if base.is_empty() || base == "/" {
        format!("/{}", name)
    } else {
        format!("{}/{}", base, name)
    }
}

fn urlencoding(s: &str) -> String {
    s.to_string()
}

fn classify_file(name: &str, is_dir: bool) -> String {
    if is_dir { return "other".to_string(); }
    let ext = Path::new(name).extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    if ["mp4", "mkv", "avi", "mov", "m2ts", "ts", "wmv", "flv"].contains(&ext.as_str()) { return "video".into(); }
    if ["jpg", "jpeg", "png", "gif", "webp", "heic", "bmp"].contains(&ext.as_str()) { return "image".into(); }
    if ["mp3", "flac", "aac", "wav", "ogg"].contains(&ext.as_str()) { return "audio".into(); }
    if ["zip", "rar", "7z", "tar", "gz", "bz2"].contains(&ext.as_str()) { return "archive".into(); }
    if ["pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "txt", "md"].contains(&ext.as_str()) { return "document".into(); }
    "other".into()
}
