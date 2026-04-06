use settings::Db;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

#[derive(Clone)]
pub struct Database {
    pub pool: Pool<Postgres>,
}

impl Database {
    pub async fn new(db: &Db) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&db.get_url())
            .await?;

        sqlx::query(&format!("SET search_path TO {}", db.shema))
            .execute(&pool)
            .await?;

        Ok(Self { pool })
    }

   
}
