use axum::Json;
pub async fn serve_file() -> Json<serde_json::Value> { Json(serde_json::json!({"ok": true})) }
