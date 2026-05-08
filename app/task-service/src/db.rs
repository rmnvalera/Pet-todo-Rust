use dtos::todos::{TaskFilter, UpdateTaskRequest};
use entities::tasks::{Priority, Task, TaskStatus};
use settings::Db;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use uuid::Uuid;

#[derive(Clone)]
pub struct Database {
    pub pool: Pool<Postgres>,
    pub schema: String,
}

impl Database {
    pub async fn connect(db: &Db) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&db.get_url())
            .await?;
        Ok(Self {
            pool,
            schema: db.shema.clone(),
        })
    }
    pub async fn migrate(&self) -> Result<(), sqlx::Error> {
        let query_schema = format!("CREATE SCHEMA IF NOT EXISTS {};", self.schema);
        sqlx::query(&query_schema).execute(&self.pool).await?;
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    pub async fn create(
        &self,
        title: String,
        description: Option<String>,
        priority: Option<Priority>,
        owner_id: Uuid,
    ) -> Result<Task, sqlx::Error> {
        let task = sqlx::query_as::<_, Task>(
            r#"
                INSERT INTO tasks (title, description, status, priority, owner_id)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING *
                "#,
        )
        .bind(title)
        .bind(description)
        .bind(TaskStatus::Todo)
        .bind(priority.unwrap_or(Priority::Medium))
        .bind(owner_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(task)
    }

    pub async fn get_by(&self, id: &Uuid) -> Result<Task, sqlx::Error> {
        let task = sqlx::query_as::<_, Task>(
            r#"
                SELECT * FROM tasks WHERE id = $1
                "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(task)
    }

    pub async fn get_all_by(
        &self,
        owner_id: &Uuid,
        filter: &TaskFilter,
    ) -> Result<(Vec<Task>, i64), sqlx::Error> {
        let offset = filter.offset();
        let per_page = filter.per_page();

        let tasks = sqlx::query_as::<_, Task>(
            r#"
                SELECT * FROM tasks
                WHERE owner_id = $1
                AND ($2::task_status IS NULL OR status = $2)
                AND ($3::task_priority IS NULL OR priority = $3)
                AND ($4::text IS NULL OR title ILIKE '%' || $4 || '%')
                ORDER BY created_at DESC
                LIMIT $5 OFFSET $6
                "#,
        )
        .bind(owner_id)
        .bind(filter.status)
        .bind(filter.priority)
        .bind(filter.search.clone())
        .bind(per_page as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await?;

        let total: i64 = sqlx::query_scalar(
            r#"
                SELECT COUNT(*) FROM tasks
                WHERE owner_id = $1
                AND ($2 IS NULL OR status = $2)
                AND ($3 IS NULL OR priority = $3)
                AND ($4 IS NULL OR title ILIKE '%' || $4 || '%')
                "#,
        )
        .bind(owner_id)
        .bind(filter.status)
        .bind(filter.priority)
        .bind(filter.search.clone())
        .fetch_one(&self.pool)
        .await?;

        Ok((tasks, total))
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), sqlx::Error> {
        let result = sqlx::query("DELETE FROM tasks WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }
        Ok(())
    }

    pub async fn update(&self, id: Uuid, data: UpdateTaskRequest) -> Result<Task, sqlx::Error> {
        let task = sqlx::query_as::<_, Task>(
            r#"
        UPDATE tasks
        SET
            title = COALESCE($2, title),
            description = COALESCE($3, description),
            status = COALESCE($4, status),
            priority = COALESCE($5, priority),
            updated_at = NOW()
        WHERE id = $1
        AND owner_id = $2
        RETURNING *
        "#,
        )
        .bind(id)
        .bind(data.title)
        .bind(data.description)
        .bind(data.status)
        .bind(data.priority)
        .fetch_one(&self.pool)
        .await?;

        Ok(task)
    }
}
