use std::sync::Arc;

use axum::{Json, extract::State};
use bcrypt::{DEFAULT_COST, hash};
use dtos::users::{RegisterRequest, UserResponse};
use errors::AppError;

use crate::AppState;

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<UserResponse>, AppError> {
    let password_hash =
        hash(&payload.password, DEFAULT_COST).map_err(|_| AppError::InternalError)?;

    // ? auto call From<sqlx::Error> for AppError
    let user = state
        .db
        .create_user(&payload.email, &payload.name, &password_hash)
        .await?;

    Ok(Json(user.into()))
}
