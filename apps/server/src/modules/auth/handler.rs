use axum::Json;
pub async fn login() -> Json<serde_json::Value> { Json(serde_json::json!({"ok": true})) }
pub async fn logout() -> Json<serde_json::Value> { Json(serde_json::json!({"ok": true})) }
pub async fn me() -> Json<serde_json::Value> { Json(serde_json::json!({"ok": true})) }
