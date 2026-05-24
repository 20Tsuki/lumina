use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(FromRow, Serialize)]
pub struct DownloadTask {
    pub id: i64,
    pub url: String,
    pub save_path: String,
    pub file_name: Option<String>,
    pub progress: f64,
    pub speed: i64,
    pub size: i64,
    pub eta: i64,
    pub status: String,
    pub error_msg: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Deserialize)]
pub struct AddDownloadRequest {
    pub url: String,
    pub save_path: String,
}
