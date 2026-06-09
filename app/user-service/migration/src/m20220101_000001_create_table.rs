use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared("CREATE SCHEMA IF NOT EXISTS user_service")
            .await?;

        manager
            .create_table(
                Table::create()
                    .table("users")
                    .if_not_exists()
                    .col(pk_uuid("id").default(Expr::cust("gen_random_uuid()")))
                    .col(string_uniq("email"))
                    .col(string("name"))
                    .col(string("password_hash"))
                    .col(timestamp_with_time_zone("created_at"))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared("DROP SCHEMA user_service").await?;

        manager
            .drop_table(Table::drop().table("users").to_owned())
            .await
    }
}
