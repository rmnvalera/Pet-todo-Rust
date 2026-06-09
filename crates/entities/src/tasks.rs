use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use uuid::Uuid;

#[sea_orm::model]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "tasks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,

    pub status: TaskStatus,
    pub priority: Priority,
    pub owner_id: Uuid,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(
    Clone, Debug, Display, PartialEq, Eq, Deserialize, Serialize, EnumIter, DeriveActiveEnum,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "task_priority")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Priority {
    #[sea_orm(string_value = "low")]
    Low,
    #[sea_orm(string_value = "medium")]
    Medium,
    #[sea_orm(string_value = "hight")]
    High,
}

#[derive(
    Clone, Debug, Display, PartialEq, Eq, Deserialize, Serialize, EnumIter, DeriveActiveEnum,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "task_status")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum TaskStatus {
    #[sea_orm(string_value = "todo")]
    Todo,
    #[sea_orm(string_value = "in_progress")]
    InProgress,
    #[sea_orm(string_value = "done")]
    Done,
}
