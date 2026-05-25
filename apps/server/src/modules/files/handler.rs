use axum::{
    body::Body,
    extract::{Multipart, Query, State},
    http::header,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;

use crate::app::AppState;
use crate::error::AppError;
use crate::modules::files::service;

#[derive(Deserialize)]
pub struct ListQuery {
    pub path: Option<String>,
}

pub async fn list(
    State(state): State<Arc<AppState>>,
    _claims: crate::middleware::auth::AuthClaims,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<crate::models::file::FileEntry>>, AppError> {
    let root = get_media_root(&state);
    let entries = service::list_files(&root, &q.path.unwrap_or_else(|| "/".into()))?;
    Ok(Json(entries))
}

fn get_media_root(_state: &Arc<AppState>) -> PathBuf {
    PathBuf::from("/")
}

#[derive(Deserialize)]
pub struct MkdirRequest {
    pub path: String,
    pub name: String,
}

pub async fn mkdir(
    State(state): State<Arc<AppState>>,
    _claims: crate::middleware::auth::AuthClaims,
    Json(req): Json<MkdirRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = get_media_root(&state);
    service::mkdir(&root, &req.path, &req.name)?;
    Ok(Json(serde_json::json!({"ok": true})))
}

#[derive(Deserialize)]
pub struct MoveRequest {
    pub from: String,
    pub to: String,
}

pub async fn move_file(
    State(state): State<Arc<AppState>>,
    _claims: crate::middleware::auth::AuthClaims,
    Json(req): Json<MoveRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = get_media_root(&state);
    service::move_file(&root, &req.from, &req.to)?;
    Ok(Json(serde_json::json!({"ok": true})))
}

#[derive(Deserialize)]
pub struct DeleteRequest {
    pub path: String,
}

pub async fn delete(
    State(state): State<Arc<AppState>>,
    _claims: crate::middleware::auth::AuthClaims,
    Json(req): Json<DeleteRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = get_media_root(&state);
    service::delete(&root, &req.path)?;
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn upload(
    State(state): State<Arc<AppState>>,
    _claims: crate::middleware::auth::AuthClaims,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = get_media_root(&state);
    let mut dest_path = String::from("/");

    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::BadRequest(e.to_string()))? {
        let name = field.name().unwrap_or("").to_string();
        if name == "path" {
            dest_path = field.text().await.unwrap_or_default();
            continue;
        }
        if name == "file" {
            let file_name = field.file_name().unwrap_or("unnamed").to_string();
            let data = field.bytes().await.map_err(|e| AppError::Internal(e.to_string()))?;
            let _target = PathBuf::from(&dest_path).join(&file_name);
            let resolved = if dest_path == "/" { root.join(&file_name) } else { root.join(dest_path.trim_start_matches('/')).join(&file_name) };
            fs::write(&resolved, &data).await.map_err(|e| AppError::Internal(e.to_string()))?;
            return Ok(Json(serde_json::json!({"ok": true, "path": format!("/{}", file_name)})));
        }
    }

    Err(AppError::BadRequest("no file field".into()))
}

#[derive(Deserialize)]
pub struct DownloadQuery {
    pub path: String,
}

pub async fn download(
    State(state): State<Arc<AppState>>,
    Query(q): Query<DownloadQuery>,
) -> Result<impl IntoResponse, AppError> {
    let root = get_media_root(&state);
    let file_path = service::download_path(&root, &q.path)?;
    let content_type = mime_guess::from_path(&file_path).first_or_octet_stream();
    let file = tokio::fs::File::open(&file_path).await.map_err(|e| AppError::NotFound(e.to_string()))?;
    let stream = tokio_util::io::ReaderStream::new(file);
    let body = Body::from_stream(stream);
    Ok(([(header::CONTENT_TYPE, content_type.to_string())], body))
}

#[derive(Deserialize)]
pub struct ThumbnailQuery {
    pub path: String,
    pub size: Option<u32>,
}

pub async fn thumbnail(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ThumbnailQuery>,
) -> Result<impl IntoResponse, AppError> {
    // If path is a bare filename (no separators), it's a pre-generated media
    // thumbnail stored in thumb_dir, not a filesystem path.
    let thumb = if !q.path.contains('/') && !q.path.contains('\\') {
        let direct = state.config.thumbnail_dir().join(&q.path);
        if direct.exists() {
            direct
        } else {
            let root = get_media_root(&state);
            service::generate_thumbnail(&root, &q.path, q.size.unwrap_or(256), state.config.thumbnail_dir())?
        }
    } else {
        let root = get_media_root(&state);
        service::generate_thumbnail(&root, &q.path, q.size.unwrap_or(256), state.config.thumbnail_dir())?
    };
    let file = tokio::fs::File::open(&thumb).await.map_err(|e| AppError::NotFound(e.to_string()))?;
    let stream = tokio_util::io::ReaderStream::new(file);
    Ok(([(header::CONTENT_TYPE, "image/jpeg")], Body::from_stream(stream)))
}
