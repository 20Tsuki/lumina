use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,
    pub username: String,
    pub role: String,
    pub exp: usize,
}

pub async fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(e.to_string()))?
        .to_string();
    Ok(hash)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

pub fn create_token(user_id: i64, username: &str, role: &str, secret: &str, expiry_hours: u32) -> Result<String, AppError> {
    let exp = Utc::now()
        .checked_add_signed(chrono::Duration::hours(expiry_hours as i64))
        .ok_or(AppError::Internal("timestamp overflow".into()))?
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        username: username.to_string(),
        role: role.to_string(),
        exp,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized("invalid token".into()))
}

pub async fn create_user(pool: &SqlitePool, username: &str, password: &str, role: &str) -> Result<i64, AppError> {
    let hash = hash_password(password).await?;
    let now = Utc::now().timestamp_millis();
    let id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO users (username, password_hash, role, created_at) VALUES (?, ?, ?, ?) RETURNING id"
    )
    .bind(username)
    .bind(&hash)
    .bind(role)
    .bind(now)
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref db_err) if db_err.is_unique_violation() => {
            AppError::Conflict("username already exists".into())
        }
        _ => AppError::Internal(e.to_string()),
    })?;
    Ok(id)
}
