use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use strum_macros::Display;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
#[allow(unused)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,

    pub status: TaskStatus,
    pub priority: Priority,
    pub owner_id: Uuid,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::Type, Serialize, Deserialize, Clone, Copy, Display)]
#[sqlx(type_name = "task_priority", rename_all = "snake_case")]
pub enum Priority {
    Low,
    Medium,
    High,
}

#[derive(Debug, sqlx::Type, Serialize, Deserialize, Clone, Copy, Display)]
#[sqlx(type_name = "task_status", rename_all = "snake_case")]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
}
