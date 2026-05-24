use axum::Json;
pub async fn list() -> Json<serde_json::Value> { Json(serde_json::json!([])) }
pub async fn mkdir() -> Json<serde_json::Value> { Json(serde_json::json!({"ok": true})) }
pub async fn move_file() -> Json<serde_json::Value> { Json(serde_json::json!({"ok": true})) }
pub async fn delete() -> Json<serde_json::Value> { Json(serde_json::json!({"ok": true})) }
pub async fn upload() -> Json<serde_json::Value> { Json(serde_json::json!({"ok": true})) }
pub async fn download() -> Json<serde_json::Value> { Json(serde_json::json!({"ok": true})) }
pub async fn thumbnail() -> Json<serde_json::Value> { Json(serde_json::json!({"ok": true})) }
