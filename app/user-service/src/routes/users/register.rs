use std::sync::Arc;

use axum::{Json, extract::State};
use bcrypt::{DEFAULT_COST, hash};
use chrono::Utc;
use dtos::users::{RegisterRequest, UserResponse};
use entities::users::ActiveModel as UserActiveModel;
use errors::AppError;
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::{NotSet, Set},
};

use crate::AppState;

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<UserResponse>, AppError> {
    let password_hash =
        hash(&payload.password, DEFAULT_COST).map_err(|_| AppError::InternalError)?;

    let user = UserActiveModel {
        id: NotSet,
        email: Set(payload.email),
        name: Set(payload.name),
        password_hash: Set(password_hash),
        created_at: Set(Utc::now()),
    };

    let user = user.insert(&state.db).await?;

    Ok(Json(user.into()))
}
