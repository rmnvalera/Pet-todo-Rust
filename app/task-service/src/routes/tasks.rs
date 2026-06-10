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
use entities::tasks::ActiveModel as TaskActiveModel;
use entities::tasks::{Column, Entity as Task};
use errors::AppError;
use extractors::{auth_user::AuthUser, validate_json::ValidatedJson};
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::{NotSet, Set},
    ColumnTrait, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter, QuerySelect, QueryTrait,
};
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

    let task = TaskActiveModel {
        id: NotSet,
        title: Set(payload.title),
        description: Set(payload.description),
        status: Set(entities::tasks::TaskStatus::Todo),
        priority: Set(payload
            .priority
            .unwrap_or(entities::tasks::Priority::Medium)),
        owner_id: Set(user.user_id),
        created_at: NotSet,
        updated_at: NotSet,
    };
    let task = task.insert(&state.db).await?;

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

    let task = Task::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

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

    let offset = filter.offset();
    let per_page = filter.per_page();
    let page = filter.page();

    let task_query = Task::find()
        .filter(Column::OwnerId.eq(user.user_id))
        .apply_if(filter.status, |q, v| q.filter(Column::Status.eq(v)))
        .apply_if(filter.priority, |q, v| q.filter(Column::Priority.eq(v)))
        .apply_if(filter.search.clone(), |q, v| {
            q.filter(Column::Title.contains(&v))
        });
    let total = task_query.clone().count(&state.db).await?;

    let tasks = task_query
        .order_by_id_desc()
        .offset(Some(offset as u64))
        .limit(Some(per_page as u64))
        .all(&state.db)
        .await?;

    Ok(Json(PaginatedResponse::<TaskResponse> {
        data: tasks.into_iter().map(Into::into).collect(),
        page,
        per_page,
        total,
    }))
}

pub async fn delete(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<StatusCode, AppError> {
    let task = Task::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    if !task.owner_id.eq(&user.user_id) {
        return Err(AppError::AccessDenied);
    }
    task.delete(&state.db).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn update(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    user: AuthUser,
    ValidatedJson(payload): ValidatedJson<UpdateTaskRequest>,
) -> Result<Json<TaskResponse>, AppError> {
    let task = Task::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;
    if !task.owner_id.eq(&user.user_id) {
        return Err(AppError::AccessDenied);
    }
    let mut task: TaskActiveModel = task.into();
    if let Some(t) = payload.title {
        task.title = Set(t);
    }
    if let Some(s) = payload.status {
        task.status = Set(s);
    }
    if let Some(p) = payload.priority {
        task.priority = Set(p);
    }
    task.description = Set(payload.description);
    let task = task.update(&state.db).await?;
    Ok(Json(task.into()))
}
