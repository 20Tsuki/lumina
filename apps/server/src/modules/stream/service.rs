use axum::{
    body::Body,
    http::{header, StatusCode},
    response::Response,
};
use std::path::{Path, PathBuf};

use crate::error::AppError;
use crate::models::file::IndexedFile;

pub struct FileRange {
    pub path: PathBuf,
    pub content_type: String,
    pub size: u64,
}

pub async fn prepare_range(pool: &sqlx::SqlitePool, id: i64) -> Result<FileRange, AppError> {
    let media =
        sqlx::query_as::<_, IndexedFile>("SELECT * FROM indexed_files WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or(AppError::NotFound("file not found".into()))?;

    let lib = sqlx::query_as::<_, (String,)>( "SELECT path FROM libraries WHERE id = ?")
        .bind(media.library_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::NotFound("library not found".into()))?;

    let full_path = Path::new(&lib.0).join(&media.file_path);
    let metadata = tokio::fs::metadata(&full_path)
        .await
        .map_err(|e| AppError::NotFound(format!("file: {}", e)))?;

    let content_type = mime_guess::from_path(&full_path)
        .first_or_octet_stream()
        .to_string();

    Ok(FileRange {
        path: full_path,
        content_type,
        size: metadata.len(),
    })
}

pub async fn serve_range(
    file_range: &FileRange,
    range_header: Option<&str>,
) -> Result<Response, AppError> {
    use std::io::SeekFrom;
    use tokio::io::{AsyncReadExt, AsyncSeekExt};

    let mut file = tokio::fs::File::open(&file_range.path)
        .await
        .map_err(|e| AppError::NotFound(format!("open file: {}", e)))?;

    let file_size = file_range.size;

    if let Some(range_str) = range_header {
        if let Some(range) = parse_range(range_str, file_size) {
            let (start, end) = range;
            let length = end - start + 1;

            file.seek(SeekFrom::Start(start))
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;

            let mut buf = vec![0u8; length as usize];
            file.read_exact(&mut buf)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;

            return Ok(Response::builder()
                .status(StatusCode::PARTIAL_CONTENT)
                .header(header::CONTENT_TYPE, &file_range.content_type)
                .header(
                    header::CONTENT_RANGE,
                    format!("bytes {}-{}/{}", start, end, file_size),
                )
                .header(header::CONTENT_LENGTH, length.to_string())
                .header(header::ACCEPT_RANGES, "bytes")
                .body(Body::from(buf))
                .unwrap());
        }
    }

    let stream = tokio_util::io::ReaderStream::new(file);
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, &file_range.content_type)
        .header(header::CONTENT_LENGTH, file_size.to_string())
        .header(header::ACCEPT_RANGES, "bytes")
        .body(Body::from_stream(stream))
        .unwrap())
}

fn parse_range(range_str: &str, file_size: u64) -> Option<(u64, u64)> {
    let range_str = range_str.strip_prefix("bytes=")?;
    let parts: Vec<&str> = range_str.split('-').collect();
    if parts.len() != 2 {
        return None;
    }

    let start: u64 = parts[0].parse().ok()?;
    let end: u64 = if parts[1].is_empty() {
        file_size - 1
    } else {
        parts[1].parse().ok()?
    };

    if start > end || end >= file_size {
        return None;
    }
    Some((start, end))
}

#[async_trait::async_trait]
pub trait TranscodeService: Send + Sync {
    async fn transcode(&self, input: &Path, format: &str) -> Result<PathBuf, AppError>;
}

pub struct NoopTranscode;

#[async_trait::async_trait]
impl TranscodeService for NoopTranscode {
    async fn transcode(&self, _input: &Path, _format: &str) -> Result<PathBuf, AppError> {
        Err(AppError::Internal(
            "transcoding not supported in v1".into(),
        ))
    }
}
