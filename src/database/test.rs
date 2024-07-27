use crate::{database::Database, models::subscriptions::DeletionStatus};

#[derive(Clone, Copy)]
pub enum TestData {
    User,
    Data,
    UserData,
}

// TODO: convert these to direct database calls
impl Database {
    pub const USER_ID: i64 = 56631;
    pub const TOKEN: &'static str = "asdfghjklqwertyuiopzxcvbnm";

    pub const SUBSCRIPTION_MISSING_GUID: uuid::Uuid =
        uuid::uuid!("d78dfb54-7c24-5b30-a127-122bf249f25a");

    pub const DELETION_PENDING_ID: i64 = 1;
    pub const DELETION_SUCCESS_ID: i64 = 2;
    pub const DELETION_FAILURE_ID: i64 = 3;
    pub const DELETION_MISSING_ID: i64 = 16;

    pub async fn new_test(args: TestData) -> anyhow::Result<Self> {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        let now = time::OffsetDateTime::now_utc();

        if matches!(args, TestData::User | TestData::UserData) {
            let hash: Vec<u8> = vec![];

            let expires = now + time::Duration::days(7 * 3);

            sqlx::query!(
                "INSERT INTO users(id, username, email, password_hash) VALUES (?, ?, ?, ?)",
                Self::USER_ID,
                "example",
                "example@example.com",
                hash,
            )
            .execute(&pool)
            .await?;

            sqlx::query!(
                "INSERT INTO user_sessions(user_id, token, expires) VALUES (?, ?, ?)",
                Self::USER_ID,
                Self::TOKEN,
                expires,
            )
            .execute(&pool)
            .await?;
        }

        if matches!(args, TestData::Data | TestData::UserData) {
            Self::subscription_1(&pool).await?;
            Self::subscription_2(&pool, now).await?;
            Self::subscription_3(&pool, now).await?;

            sqlx::query!(
                r#"--sql
                    INSERT INTO task_deletions(id, user_id, subscription_id, status)
                    VALUES (?, ?, ?, ?)
                "#,
                Self::DELETION_PENDING_ID,
                Self::USER_ID,
                Self::SUBSCRIPTION_1_ID,
                DeletionStatus::Pending,
            )
            .execute(&pool)
            .await?;

            sqlx::query!(
                r#"--sql
                    INSERT INTO task_deletions(id, user_id, subscription_id, status)
                    VALUES (?, ?, ?, ?)
                "#,
                Self::DELETION_SUCCESS_ID,
                Self::USER_ID,
                Self::SUBSCRIPTION_2_ID,
                DeletionStatus::Success,
            )
            .execute(&pool)
            .await?;

            sqlx::query!(
                r#"--sql
                    INSERT INTO task_deletions(id, user_id, subscription_id, status)
                    VALUES (?, ?, ?, ?)
                "#,
                Self::DELETION_FAILURE_ID,
                Self::USER_ID,
                Self::SUBSCRIPTION_3_ID,
                DeletionStatus::Failure,
            )
            .execute(&pool)
            .await?;
        }

        Ok(Self { pool })
    }

    pub const SUBSCRIPTION_1_ID: i64 = 80890;
    pub const SUBSCRIPTION_1_FEED: &'static str = "http://one.example.com/feed.rss";
    pub const SUBSCRIPTION_1_GUID: uuid::Uuid = uuid::uuid!("1c736505-c5e0-5b9d-94cd-dcb383069b49");

    async fn subscription_1(pool: &sqlx::sqlite::SqlitePool) -> anyhow::Result<()> {
        sqlx::query!(
            "INSERT INTO subscriptions (id) VALUES (?)",
            Self::SUBSCRIPTION_1_ID,
        )
        .execute(&*pool)
        .await?;

        sqlx::query!(
            "INSERT INTO user_subscriptions (user_id, subscription_id) VALUES (?, ?)",
            Self::USER_ID,
            Self::SUBSCRIPTION_1_ID,
        )
        .execute(&*pool)
        .await?;

        sqlx::query!(
            "INSERT INTO subscription_feeds (subscription_id, feed) VALUES (?, ?)",
            Self::SUBSCRIPTION_1_ID,
            Self::SUBSCRIPTION_1_FEED,
        )
        .execute(&*pool)
        .await?;

        sqlx::query!(
            "INSERT INTO subscription_guids (subscription_id, guid) VALUES (?, ?)",
            Self::SUBSCRIPTION_1_ID,
            Self::SUBSCRIPTION_1_GUID,
        )
        .execute(&*pool)
        .await?;

        Ok(())
    }

    pub const SUBSCRIPTION_2_ID: i64 = 92766;
    pub const SUBSCRIPTION_2_FEED_OLD: &'static str = "http://two-old.example.com/feed.rss";
    pub const SUBSCRIPTION_2_FEED_NEW: &'static str = "http://two-new.example.com/feed.rss";
    pub const SUBSCRIPTION_2_GUID: uuid::Uuid = uuid::uuid!("7f3f76e4-79d1-5a05-8d21-5438d032fdd6");

    async fn subscription_2(
        pool: &sqlx::sqlite::SqlitePool,
        now: time::OffsetDateTime,
    ) -> anyhow::Result<()> {
        let then = now - time::Duration::days(7 * 3);

        sqlx::query!(
            "INSERT INTO subscriptions (id) VALUES (?)",
            Self::SUBSCRIPTION_2_ID,
        )
        .execute(&*pool)
        .await?;

        sqlx::query!(
            "INSERT INTO user_subscriptions (user_id, subscription_id) VALUES (?, ?)",
            Self::USER_ID,
            Self::SUBSCRIPTION_2_ID,
        )
        .execute(&*pool)
        .await?;

        sqlx::query!(
            "INSERT INTO subscription_feeds (subscription_id, feed, created) VALUES (?, ?, ?)",
            Self::SUBSCRIPTION_2_ID,
            Self::SUBSCRIPTION_2_FEED_OLD,
            then,
        )
        .execute(&*pool)
        .await?;
        sqlx::query!(
            "INSERT INTO subscription_feeds (subscription_id, feed, created) VALUES (?, ?, ?)",
            Self::SUBSCRIPTION_2_ID,
            Self::SUBSCRIPTION_2_FEED_NEW,
            now,
        )
        .execute(&*pool)
        .await?;

        sqlx::query!(
            "INSERT INTO subscription_guids (subscription_id, guid) VALUES (?, ?)",
            Self::SUBSCRIPTION_2_ID,
            Self::SUBSCRIPTION_2_GUID,
        )
        .execute(&*pool)
        .await?;

        Ok(())
    }

    pub const SUBSCRIPTION_3_ID: i64 = 37239;
    pub const SUBSCRIPTION_3_FEED: &'static str = "http://three.example.com/feed.rss";
    pub const SUBSCRIPTION_3_GUID_OLD: uuid::Uuid =
        uuid::uuid!("cbfab27c-5529-5fe2-a7e1-607bdb128145");
    pub const SUBSCRIPTION_3_GUID_NEW: uuid::Uuid =
        uuid::uuid!("8056e44f-978e-44b3-b34a-e99c79b6d891");

    async fn subscription_3(
        pool: &sqlx::sqlite::SqlitePool,
        now: time::OffsetDateTime,
    ) -> anyhow::Result<()> {
        let then = now - time::Duration::days(7 * 3);

        sqlx::query!(
            "INSERT INTO subscriptions (id) VALUES (?)",
            Self::SUBSCRIPTION_3_ID,
        )
        .execute(&*pool)
        .await?;

        sqlx::query!(
            "INSERT INTO user_subscriptions (user_id, subscription_id) VALUES (?, ?)",
            Self::USER_ID,
            Self::SUBSCRIPTION_3_ID,
        )
        .execute(&*pool)
        .await?;

        sqlx::query!(
            "INSERT INTO subscription_feeds (subscription_id, feed) VALUES (?, ?)",
            Self::SUBSCRIPTION_3_ID,
            Self::SUBSCRIPTION_3_FEED,
        )
        .execute(&*pool)
        .await?;

        sqlx::query!(
            "INSERT INTO subscription_guids (subscription_id, guid, created) VALUES (?, ?, ?)",
            Self::SUBSCRIPTION_3_ID,
            Self::SUBSCRIPTION_3_GUID_OLD,
            then,
        )
        .execute(&*pool)
        .await?;
        sqlx::query!(
            "INSERT INTO subscription_guids (subscription_id, guid, created) VALUES (?, ?, ?)",
            Self::SUBSCRIPTION_3_ID,
            Self::SUBSCRIPTION_3_GUID_NEW,
            now,
        )
        .execute(&*pool)
        .await?;

        Ok(())
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
