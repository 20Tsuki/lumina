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

impl<S> FromRequestParts<S> for AuthClaims
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    fn from_request_parts<'life0, 'life1, 'async_trait>(
        parts: &'life0 mut Parts,
        _state: &'life1 S,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self, Self::Rejection>> + Send + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
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

            let state = parts.extensions.get::<Arc<AppState>>().ok_or_else(|| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": {"code": "INTERNAL", "message": "state missing"}})),
                )
            })?;

            let claims = service::verify_token(header, &state.config.jwt_secret()).map_err(
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
        })
    }
}
