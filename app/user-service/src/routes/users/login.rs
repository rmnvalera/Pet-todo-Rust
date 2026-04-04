use std::sync::Arc;

use axum::{Json, extract::State};
use chrono::Duration;
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::Utc;

use crate::{
    AppState,
    dto::users::{JwtResponse, LoginRequest},
    errors::AppError,
};

#[derive(Debug, Serialize, Deserialize)]
#[allow(unused)]
pub struct Claims {
    pub sub: String, // user id
    pub email: String,
    exp: usize, // expiration timestamp
}

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
        exp: (Utc::now() + Duration::hours(settings.auth.jwt_deadline.as_secs() as i64)).timestamp()
            as usize,
    };

    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claim,
        &jsonwebtoken::EncodingKey::from_secret(settings.auth.jwt_secret.as_ref()),
    )
    .map_err(|_| AppError::InternalError)?;

    Ok(Json(JwtResponse {
        token,
        user: user.into(),
    }))
}
