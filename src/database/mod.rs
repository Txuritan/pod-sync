pub mod subscription;
pub mod tasks;

pub mod session;
#[cfg(test)]
pub mod test;
pub mod user;

#[derive(Clone)]
pub struct Database {
    pub pool: sqlx::SqlitePool,
}

impl Database {
    pub async fn new() -> anyhow::Result<Self> {
        let pool = sqlx::SqlitePool::connect("sqlite://pod-sync.db").await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        sqlx::query!("PRAGMA foreign_keys = ON;")
            .execute(&pool)
            .await?;

        Ok(Self { pool })
    }

    pub async fn shutdown(&self) -> anyhow::Result<()> {
        self.pool.close().await;

        Ok(())
    }
}
