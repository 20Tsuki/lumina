use serde::Serialize;
use sqlx::FromRow;

#[derive(FromRow, Serialize, Clone)]
pub struct IndexedFile {
    pub id: i64,
    pub library_id: i64,
    pub file_path: String,
    pub file_type: String,
    pub title: String,
    pub size: i64,
    pub codec: Option<String>,
    pub resolution: Option<String>,
    pub duration: Option<i64>,
    pub bitrate: Option<i64>,
    pub thumb_path: Option<String>,
    pub metadata_json: Option<String>,
    pub status: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(FromRow, Serialize)]
pub struct Library {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub library_type: String,
    pub created_at: i64,
}

#[derive(Serialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub file_type: String,
    pub size: i64,
    pub is_dir: bool,
    pub modified_at: i64,
    pub thumbnail_url: Option<String>,
}

#[derive(Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
    pub total: i64,
}
