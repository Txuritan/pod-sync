pub mod session;
pub mod subscription;
pub mod user;

use crate::error::Result;

#[cfg(test)]
#[derive(Clone, Copy)]
pub enum TestData {
    User,
    Data,
    UserData,
}

#[derive(Clone)]
pub struct Database {
    pub pool: sqlx::SqlitePool,
}

impl Database {
    #[cfg(test)]
    pub const TEST_TOKEN: &'static str = "asdfghjklqwertyuiopzxcvbnm";

    pub async fn new() -> Result<Self> {
        let pool = sqlx::SqlitePool::connect("sqlite://pod-sync.db").await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        sqlx::query!("PRAGMA foreign_keys = ON;")
            .execute(&pool)
            .await?;

        Ok(Self { pool })
    }

    #[cfg(test)]
    pub async fn new_test(args: TestData) -> Result<Self> {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        if matches!(args, TestData::User | TestData::UserData) {
            let hash: Vec<u8> = vec![];

            let now = time::OffsetDateTime::now_utc();
            let expires = now + time::Duration::days(7 * 3);

            // TODO: add test data
            sqlx::query!(
                "INSERT INTO users(id, username, email, password_hash) VALUES (?, ?, ?, ?)",
                123456,
                "example",
                "example@example.com",
                hash,
            )
            .execute(&pool)
            .await?;

            sqlx::query!(
                "INSERT INTO user_sessions(user_id, token, expires) VALUES (?, ?, ?)",
                123456,
                Self::TEST_TOKEN,
                expires
            )
            .execute(&pool)
            .await?;
        }

        if matches!(args, TestData::Data | TestData::UserData) {}

        Ok(Self { pool })
    }

    pub async fn shutdown(&self) -> Result<()> {
        self.pool.close().await;

        Ok(())
    }
}
