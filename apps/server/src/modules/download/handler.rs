use axum::Json;
pub async fn add() -> Json<serde_json::Value> { Json(serde_json::json!({"ok": true})) }
pub async fn list() -> Json<serde_json::Value> { Json(serde_json::json!([])) }
pub async fn pause() -> Json<serde_json::Value> { Json(serde_json::json!({"ok": true})) }
pub async fn resume() -> Json<serde_json::Value> { Json(serde_json::json!({"ok": true})) }
pub async fn remove() -> Json<serde_json::Value> { Json(serde_json::json!({"ok": true})) }
pub async fn progress_sse() -> Json<serde_json::Value> { Json(serde_json::json!({"ok": true})) }
