pub mod subscription;
pub mod tasks;

pub mod session;
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

#[cfg(test)]
impl Database {
    pub const USER_ID: i64 = 56631;
    pub const TOKEN: &'static str = "asdfghjklqwertyuiopzxcvbnm";

    pub const SUBSCRIPTION_MISSING_GUID: uuid::Uuid =
        uuid::uuid!("d78dfb54-7c24-5b30-a127-122bf249f25a");

    pub const DELETION_PENDING_ID: i64 = 1;
    pub const DELETION_SUCCESS_ID: i64 = 2;
    pub const DELETION_FAILURE_ID: i64 = 3;
    pub const DELETION_MISSING_ID: i64 = 16;

    pub const SUBSCRIPTION_1_ID: i64 = 80890;
    pub const SUBSCRIPTION_1_FEED: &'static str = "http://one.example.com/feed.rss";
    pub const SUBSCRIPTION_1_GUID: uuid::Uuid = uuid::uuid!("1c736505-c5e0-5b9d-94cd-dcb383069b49");

    pub const SUBSCRIPTION_2_ID: i64 = 92766;
    pub const SUBSCRIPTION_2_FEED_OLD: &'static str = "http://two-old.example.com/feed.rss";
    pub const SUBSCRIPTION_2_FEED_NEW: &'static str = "http://two-new.example.com/feed.rss";
    pub const SUBSCRIPTION_2_GUID: uuid::Uuid = uuid::uuid!("7f3f76e4-79d1-5a05-8d21-5438d032fdd6");

    pub const SUBSCRIPTION_3_ID: i64 = 37239;
    pub const SUBSCRIPTION_3_FEED: &'static str = "http://three.example.com/feed.rss";
    pub const SUBSCRIPTION_3_GUID_OLD: uuid::Uuid =
        uuid::uuid!("cbfab27c-5529-5fe2-a7e1-607bdb128145");
    pub const SUBSCRIPTION_3_GUID_NEW: uuid::Uuid =
        uuid::uuid!("8056e44f-978e-44b3-b34a-e99c79b6d891");

    pub async fn new_test(pool: sqlx::SqlitePool) -> anyhow::Result<Self> {
        Ok(Self { pool })
    }

    #[track_caller]
    pub fn test_token() -> headers::HeaderValue {
        use headers::{
            authorization::{Bearer, Credentials as _},
            Authorization,
        };

        Authorization::<Bearer>::bearer(Self::TOKEN)
            .unwrap()
            .0
            .encode()
    }
}

