use serde::Serialize;
use sqlx::FromRow;

#[derive(FromRow)]
pub struct UserRow {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub role: String,
    pub created_at: i64,
}

#[derive(Serialize)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
    pub role: String,
    pub created_at: i64,
}
