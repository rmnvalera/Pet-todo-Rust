use std::sync::Arc;

use axum::{Json, debug_handler, extract::State};
use dtos::users::UserResponse;
use entities::users::Entity as User;
use errors::AppError;
use extractors::auth_user::AuthUser;
use sea_orm::EntityTrait;

use crate::AppState;

#[debug_handler]
pub async fn handler(
    AuthUser { user_id, .. }: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<UserResponse>, AppError> {
    let user = User::find_by_id(user_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(user.into()))
}
