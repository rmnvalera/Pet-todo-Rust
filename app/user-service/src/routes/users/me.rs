use std::sync::Arc;

use axum::{Json, extract::State};

use crate::{AppState, dto::users::UserResponse, errors::AppError, extractors::AuthUser};

pub async fn handler(
    AuthUser { user_id, .. }: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<UserResponse>, AppError> {
    let user = state.db.get_user_by_id(&user_id).await?;
    Ok(Json(user.into()))
}
