use sqlx::{
    Pool, Postgres,
    postgres::PgPoolOptions,
    types::chrono::{DateTime, Utc},
};
use uuid::Uuid;

#[derive(Clone)]
pub struct Database {
    pub pool: Pool<Postgres>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(unused)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

impl Database {
    pub async fn new(url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new().max_connections(5).connect(url).await?;
        Ok(Self { pool })
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
