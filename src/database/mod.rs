pub mod session;
pub mod subscription;
pub mod user;

use crate::error::Result;

#[derive(Clone)]
pub struct Database {
    pub pool: sqlx::SqlitePool,
}

impl Database {
    pub async fn new() -> Result<Self> {
        let pool = sqlx::SqlitePool::connect("sqlite://pod-sync.db").await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        sqlx::query!("PRAGMA foreign_keys = ON;")
            .execute(&pool)
            .await?;

        Ok(Self { pool })
    }

    #[cfg(test)]
    pub async fn new_test() -> Result<Self> {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        // TODO: add test data

        Ok(Self { pool })
    }

    pub async fn shutdown(&self) -> Result<()> {
        self.pool.close().await;

        Ok(())
    }
}
