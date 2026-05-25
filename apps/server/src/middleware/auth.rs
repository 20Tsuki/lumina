use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    Json,
};
use serde_json::json;
use std::sync::Arc;

use crate::app::AppState;
use crate::modules::auth::service;

#[derive(Clone, Debug)]
pub struct AuthClaims {
    pub user_id: i64,
    pub username: String,
    pub role: String,
}

impl FromRequestParts<Arc<AppState>> for AuthClaims {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": {"code": "UNAUTHORIZED", "message": "missing token"}})),
                )
            })?;

        let claims = service::verify_token(header, state.config.jwt_secret()).map_err(
            |_| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": {"code": "UNAUTHORIZED", "message": "invalid token"}})),
                )
            },
        )?;

        Ok(AuthClaims {
            user_id: claims.sub,
            username: claims.username,
            role: claims.role,
        })
    }
}
