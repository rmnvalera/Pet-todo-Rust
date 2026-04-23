use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use dtos::{
    pagination::PaginatedResponse,
    todos::{CreateTaskRequest, TaskFilter, TaskResponse, UpdateTaskRequest},
};
use errors::AppError;
use extractors::{auth_user::AuthUser, validate_json::ValidatedJson};
use uuid::Uuid;

use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create).get(get_all))
        .route("/:id", get(get_one).delete(delete).patch(update))
}

#[axum::debug_handler]
pub async fn create(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    ValidatedJson(payload): ValidatedJson<CreateTaskRequest>,
) -> Result<Json<TaskResponse>, AppError> {
    // TODO pool.begin()
    
    let task = state
        .db
        .create(
            payload.title,
            payload.description,
            payload.priority,
            user.user_id,
        )
        .await?;

    state
        .bus
        .publish(
            "task.created",
            &serde_json::to_vec(&task).map_err(|_| AppError::InternalError)?,
        )
        .await
        .map_err(|_| AppError::InternalError)?;
    // TODO pool.commit()
    Ok(Json(task.into()))
}

#[axum::debug_handler]
pub async fn get_one(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<TaskResponse>, AppError> {
    tracing::info!("one  for: {}", id);

    let task = state.db.get_by(&id).await?;
    if !task.owner_id.eq(&user.user_id) {
        return Err(AppError::AccessDenied);
    }

    Ok(Json(task.into()))
}

pub async fn get_all(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Query(filter): Query<TaskFilter>,
) -> Result<Json<PaginatedResponse<TaskResponse>>, AppError> {
    tracing::info!("All");
    let task = state.db.get_all_by(&user.user_id, &filter).await?;

    Ok(Json(PaginatedResponse::<TaskResponse> {
        data: task.0.into_iter().map(Into::into).collect(),
        page: filter.page(),
        per_page: filter.per_page(),
        total: task.1,
    }))
}

pub async fn delete(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<StatusCode, AppError> {
    let task = state.db.get_by(&id).await?;
    if !task.owner_id.eq(&user.user_id) {
        return Err(AppError::AccessDenied);
    }
    state.db.delete(task.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn update(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    user: AuthUser,
    ValidatedJson(payload): ValidatedJson<UpdateTaskRequest>,
) -> Result<Json<TaskResponse>, AppError> {
    let task = state.db.get_by(&id).await?;
    if !task.owner_id.eq(&user.user_id) {
        return Err(AppError::AccessDenied);
    }
    let task = state.db.update(id, payload).await?;
    Ok(Json(task.into()))
}
