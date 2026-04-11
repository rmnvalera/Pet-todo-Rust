use entities::users::User;
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

    pub async fn _get_all_users(&self) -> Result<Vec<User>, sqlx::Error> {
        let users = sqlx::query_as::<_, User>("SELECT * FROM users")
            .fetch_all(&self.pool)
            .await?;
        Ok(users)
    }

    pub async fn get_user_by_id(&self, id: &Uuid) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(user)
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_one(&self.pool)
            .await?;
        Ok(user)
    }

    pub async fn create_user(
        &self,
        email: &str,
        name: &str,
        password_hash: &str,
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (email, name, password_hash)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(email)
        .bind(name)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }
}
