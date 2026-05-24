use axum::Json;
pub async fn scan() -> Json<serde_json::Value> { Json(serde_json::json!({"ok": true})) }
pub async fn scan_status() -> Json<serde_json::Value> { Json(serde_json::json!({"ok": true})) }
pub async fn movies() -> Json<serde_json::Value> { Json(serde_json::json!([])) }
pub async fn series_list() -> Json<serde_json::Value> { Json(serde_json::json!([])) }
pub async fn search() -> Json<serde_json::Value> { Json(serde_json::json!([])) }
pub async fn media_detail() -> Json<serde_json::Value> { Json(serde_json::json!({})) }
pub async fn refresh() -> Json<serde_json::Value> { Json(serde_json::json!({"ok": true})) }
