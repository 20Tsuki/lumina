use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::Response,
};
use std::sync::Arc;

use crate::app::AppState;
use crate::error::AppError;
use crate::modules::stream::service;

pub async fn serve_file(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let file_range = service::prepare_range(&state.pool, id).await?;
    let range_header = headers.get("range").and_then(|v| v.to_str().ok());
    service::serve_range(&file_range, range_header).await
}
