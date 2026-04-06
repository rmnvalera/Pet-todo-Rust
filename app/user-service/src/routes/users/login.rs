use std::sync::Arc;

use auth_jwt::Claims;
use axum::{Json, extract::State};
use chrono::Duration;
use dtos::users::{JwtResponse, LoginRequest};
use errors::AppError;
use sqlx::types::chrono::Utc;

use crate::AppState;

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<JwtResponse>, AppError> {
    let settings = &state.settings;
    let user = state.db.get_user_by_email(&payload.email).await?;
    let is_valid = bcrypt::verify(&payload.password, &user.password_hash)
        .map_err(|_| AppError::InternalError)?;

    if !is_valid {
        return Err(AppError::Unauthorized);
    }

    let claim = Claims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        exp: (Utc::now() + Duration::hours(settings.jwt.deadline.as_secs() as i64)).timestamp()
            as usize,
    };

    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claim,
        &jsonwebtoken::EncodingKey::from_secret(settings.jwt.secret.as_ref()),
    )
    .map_err(|_| AppError::InternalError)?;

    Ok(Json(JwtResponse {
        token,
        user: user.into(),
    }))
}
