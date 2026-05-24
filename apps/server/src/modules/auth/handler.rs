use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::app::AppState;
use crate::error::AppError;
use crate::modules::auth::service;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: crate::models::user::UserInfo,
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let user = sqlx::query_as::<_, crate::models::user::UserRow>(
        "SELECT id, username, password_hash, role, created_at FROM users WHERE username = ?",
    )
    .bind(&req.username)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .ok_or(AppError::Unauthorized("invalid credentials".into()))?;

    if !service::verify_password(&req.password, &user.password_hash)? {
        return Err(AppError::Unauthorized("invalid credentials".into()));
    }

    let token = service::create_token(
        user.id,
        &user.username,
        &user.role,
        &state.config.jwt_secret(),
        state.config.jwt_expiry_hours(),
    )?;

    Ok(Json(LoginResponse {
        token,
        user: crate::models::user::UserInfo {
            id: user.id,
            username: user.username,
            role: user.role,
            created_at: user.created_at,
        },
    }))
}

pub async fn logout() -> &'static str {
    "ok"
}

pub async fn me(
    State(state): State<Arc<AppState>>,
    claims: crate::middleware::auth::AuthClaims,
) -> Result<Json<crate::models::user::UserInfo>, AppError> {
    let created_at = sqlx::query_scalar::<_, i64>(
        "SELECT created_at FROM users WHERE id = ?",
    )
    .bind(claims.user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .unwrap_or(0);

    Ok(Json(crate::models::user::UserInfo {
        id: claims.user_id,
        username: claims.username,
        role: claims.role,
        created_at,
    }))
}
