use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

impl ActiveModelBehavior for ActiveModel {}
