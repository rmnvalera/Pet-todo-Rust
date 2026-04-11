use std::sync::Arc;

use axum::{Json, debug_handler, extract::State};
use dtos::users::UserResponse;
use errors::AppError;
use extractors::auth_user::AuthUser;

use crate::AppState;

#[debug_handler]
pub async fn handler(
    AuthUser { user_id, .. }: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<UserResponse>, AppError> {
    let user = state.db.get_user_by_id(&user_id).await?;
    Ok(Json(user.into()))
}
