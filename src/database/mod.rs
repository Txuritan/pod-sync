pub mod session;
pub mod subscription;
pub mod user;

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

#[cfg(test)]
impl Database {
    pub const TEST_USER_ID: i64 = 123456;
    pub const TEST_TOKEN: &'static str = "asdfghjklqwertyuiopzxcvbnm";
    pub const TEST_SUBSCRIPTION_ID: i64 = 456789;
    pub const TEST_SUBSCRIPTION_GUID: uuid::Uuid = uuid::uuid!("73a2a25c-b5d6-4e70-a6c4-046c1b0f8fc2");
    pub const TEST_SUBSCRIPTION_FEED: &'static str = "http://example.com/feed.rss";

    pub async fn new_test(args: TestData) -> anyhow::Result<Self> {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        if matches!(args, TestData::User | TestData::UserData) {
            let hash: Vec<u8> = vec![];

            let now = time::OffsetDateTime::now_utc();
            let expires = now + time::Duration::days(7 * 3);

            // TODO: add test data
            sqlx::query!(
                "INSERT INTO users(id, username, email, password_hash) VALUES (?, ?, ?, ?)",
                Self::TEST_USER_ID,
                "example",
                "example@example.com",
                hash,
            )
            .execute(&pool)
            .await?;

            sqlx::query!(
                "INSERT INTO user_sessions(user_id, token, expires) VALUES (?, ?, ?)",
                Self::TEST_USER_ID,
                Self::TEST_TOKEN,
                expires,
            )
            .execute(&pool)
            .await?;
        }

        if matches!(args, TestData::Data | TestData::UserData) {
            sqlx::query!(
                "INSERT INTO subscriptions (id) VALUES (?)",
                Self::TEST_SUBSCRIPTION_ID,
            )
            .execute(&pool)
            .await?;

            sqlx::query!(
                "INSERT INTO user_subscriptions (user_id, subscription_id) VALUES (?, ?)",
                Self::TEST_USER_ID,
                Self::TEST_SUBSCRIPTION_ID,
            )
            .execute(&pool)
            .await?;

            sqlx::query!(
                "INSERT INTO subscription_feeds (subscription_id, feed) VALUES (?, ?)",
                Self::TEST_SUBSCRIPTION_ID,
                Self::TEST_SUBSCRIPTION_FEED,
            )
            .execute(&pool)
            .await?;

            sqlx::query!(
                "INSERT INTO subscription_guids (subscription_id, guid) VALUES (?, ?)",
                Self::TEST_SUBSCRIPTION_ID,
                Self::TEST_SUBSCRIPTION_GUID,
            )
            .execute(&pool)
            .await?;
        }

        Ok(Self { pool })
    }

    #[track_caller]
    pub fn test_token() -> headers::HeaderValue {
        use headers::{
            authorization::{Bearer, Credentials as _},
            Authorization,
        };

        Authorization::<Bearer>::bearer(Self::TEST_TOKEN)
            .unwrap()
            .0
            .encode()
    }
}
