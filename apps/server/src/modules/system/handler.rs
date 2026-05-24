use axum::Json;
pub async fn info() -> Json<serde_json::Value> { Json(serde_json::json!({})) }
pub async fn get_settings() -> Json<serde_json::Value> { Json(serde_json::json!({})) }
pub async fn update_settings() -> Json<serde_json::Value> { Json(serde_json::json!({"ok": true})) }
