use sea_orm_migration::{prelude::*, schema::*, sea_query::extension::postgres::Type};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared("CREATE SCHEMA IF NOT EXISTS task_service")
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum("task_status")
                    .values(["todo", "in_progrss", "done"])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum("task_priority")
                    .values(["low", "medium", "hight"])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table("tasks")
                    .if_not_exists()
                    .col(pk_uuid("id").default(Expr::cust("gen_random_uuid()")))
                    .col(string("title"))
                    .col(text("description").null())
                    .col(
                        ColumnDef::new("status")
                            .custom(Alias::new("task_status"))
                            .not_null()
                            .default("todo"),
                    )
                    .col(
                        ColumnDef::new("priority")
                            .custom(Alias::new("task_priority"))
                            .not_null()
                            .default("medium"),
                    )
                    .col(uuid("owner_id").not_null())
                    .col(timestamp_with_time_zone("created_at").default(Expr::cust("NOW()")))
                    .col(timestamp_with_time_zone("updated_at").default(Expr::cust("NOW()")))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared("DROP SCHEMA task_service").await?;

        manager
            .drop_type(Type::drop().name("task_status").to_owned())
            .await?;
        manager
            .drop_type(Type::drop().name("task_priority").to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table("task").to_owned())
            .await
    }
}
