use std::fmt;

use argon2::{
    password_hash::SaltString, Argon2, PasswordHash, PasswordHasher as _, PasswordVerifier as _,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use data_encoding::BASE64;
use rand::{rngs::OsRng, RngCore as _};
use time::OffsetDateTime;

use crate::{
    error::{Error, Result},
    extractor::auth::{Session, SESSION},
};

#[derive(sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
}

impl User {
    #[tracing::instrument(skip_all)]
    #[autometrics::autometrics]
    pub fn verify(&self, password: &str) -> bool {
        let Ok(parsed_hash) = PasswordHash::new(&self.password_hash) else {
            return false;
        };

        let argon = Argon2::default();

        argon
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    }
}

#[derive(sqlx::FromRow)]
pub struct OptionalSession {
    pub id: Option<i64>,
    pub username: Option<String>,
    pub email: Option<String>,
    pub password_hash: Option<String>,
    pub expires: Option<i64>,
}

impl OptionalSession {
    pub fn into_session(self) -> Option<Session> {
        let id = self.id?;
        let username = self.username?;
        let email = self.email?;
        let password_hash = self.password_hash?;
        let expires = self.expires?;

        Some(Session {
            expires,
            user: User {
                id,
                username,
                email,
                password_hash,
            },
        })
    }
}

#[derive(Clone)]
pub struct Database {
    pool: sqlx::SqlitePool,
}

impl Database {
    pub async fn new() -> Result<Self> {
        let pool = sqlx::SqlitePool::connect("sqlite://pod-sync.db").await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    #[cfg(test)]
    pub async fn new_test() -> Result<Self> {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        // TODO: add test data

        Ok(Self { pool })
    }

    pub async fn shutdown(&self) {
        self.pool.close().await;
    }
}

impl Database {
    #[tracing::instrument(skip_all, err)]
    #[autometrics::autometrics]
    pub async fn user_create(&self, username: &str, email: &str, password: &str) -> Result<i64> {
        struct Wrapper {
            id: i64,
        }

        // TODO: use a pepper
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)?
            .to_string();

        let wrapper = sqlx::query_as!(
            Wrapper,
            "INSERT INTO users (username, email, password_hash) VALUES (?, ?, ?) RETURNING id",
            username,
            email,
            password_hash
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(wrapper.id)
    }

    pub async fn user_login(&self, user: &User) -> Result<Cookie<'static>> {
        let now = OffsetDateTime::now_utc();
        let expires = now + time::Duration::days(7 * 3);

        let mut bytes = [0; 64];
        OsRng.fill_bytes(&mut bytes);
        let token = BASE64.encode(&bytes);

        sqlx::query!(
            "INSERT INTO user_sessions ( user_id, token, expires ) VALUES ( ?, ?, ? )",
            user.id,
            token,
            expires
        )
        .execute(&self.pool)
        .await?;

        let mut cookie = Cookie::new(SESSION, token);
        cookie.set_http_only(true);
        cookie.set_path("/");
        cookie.set_same_site(SameSite::Strict);
        cookie.set_expires(expires);

        Ok(cookie)
    }

    #[tracing::instrument(skip_all, err)]
    #[autometrics::autometrics]
    pub async fn user_get_by_id(&self, id: i64) -> Result<Option<User>> {
        sqlx::query_as!(
            User,
            "SELECT id, username, email, password_hash FROM users WHERE id = ? LIMIT 1",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::from)
    }

    #[tracing::instrument(skip_all, err)]
    #[autometrics::autometrics]
    pub async fn user_get_by_username(&self, username: &str) -> Result<Option<User>> {
        sqlx::query_as!(
            User,
            "SELECT id, username, email, password_hash FROM users WHERE username = ? LIMIT 1",
            username
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::from)
    }

    #[tracing::instrument(skip_all, err)]
    #[autometrics::autometrics]
    pub async fn user_get_session(&self, token: &str) -> Result<Option<Session>> {
        sqlx::query_as!(
            OptionalSession,
            "SELECT u.id, u.username, u.email, u.password_hash, us.expires FROM user_sessions us LEFT JOIN users u ON us.user_id = u.id WHERE us.token = ? LIMIT 1",
            token
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::from)
        .map(|ok| ok.and_then(OptionalSession::into_session))
    }
}

#[derive(Clone, serde::Serialize)]
pub struct Device {
    #[serde(skip)]
    pub true_id: i64,

    pub id: String,
    pub caption: String,
    #[serde(rename = "type")]
    pub typ: DeviceType,
    pub subscriptions: i32,
}

#[derive(Clone, serde::Deserialize)]
pub struct DeviceUpdate {
    pub caption: Option<String>,
    #[serde(rename = "type")]
    pub typ: Option<DeviceType>,
}

#[derive(Clone, Copy, Default, serde::Deserialize, serde::Serialize, sqlx::Type)]
#[repr(i32)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    Desktop,
    Laptop,
    Mobile,
    Server,
    #[default]
    Other,
}

impl fmt::Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DeviceType::Desktop => "desktop",
                DeviceType::Laptop => "laptop",
                DeviceType::Mobile => "mobile",
                DeviceType::Server => "server",
                DeviceType::Other => "other",
            }
        )
    }
}

impl Database {
    #[tracing::instrument(skip_all, err)]
    #[autometrics::autometrics]
    pub async fn get_devices(&self, user: &User) -> Result<Vec<Device>> {
        sqlx::query_as!(Device, "SELECT id as true_id, name as id, caption, type as 'typ: DeviceType', 0 as subscriptions FROM devices WHERE user_id = ?", user.id)
            .fetch_all(&self.pool)
            .await
            .map_err(Error::from)
    }

    #[tracing::instrument(skip_all, err)]
    #[autometrics::autometrics]
    pub async fn get_device_id(&self, user: &User, device_name: &str) -> Result<Option<i64>> {
        struct Wrapper {
            id: i64,
        }

        sqlx::query_as!(
            Wrapper,
            "SELECT id FROM devices WHERE user_id = ? AND name = ?",
            user.id,
            device_name
        )
        .fetch_optional(&self.pool)
        .await
        .map(|w| w.map(|w| w.id))
        .map_err(Error::from)
    }

    #[tracing::instrument(skip_all, err)]
    #[autometrics::autometrics]
    pub async fn update_device(
        &self,
        user: &User,
        name: &str,
        update: DeviceUpdate,
    ) -> Result<i64> {
        struct Wrapper {
            id: i64,
        }

        sqlx::query_as!(Wrapper, "INSERT INTO devices (user_id, name, caption, type) VALUES (?, ?, ?, ?) ON CONFLICT(user_id, name) DO UPDATE SET user_id=excluded.user_id,name=excluded.name,caption=excluded.caption,type=excluded.type RETURNING id", user.id, name, update.caption, update.typ)
            .fetch_one(&self.pool)
            .await
            .map(|w| w.id)
            .map_err(Error::from)
    }

    #[tracing::instrument(skip_all, err)]
    #[autometrics::autometrics]
    pub async fn remove_device(&self, user: &User, device_id: i64) -> Result<()> {
        let mut trans = self.pool.begin().await?;

        sqlx::query!(
            "DELETE FROM devices WHERE user_id = ? AND id = ?",
            user.id,
            device_id
        )
        .execute(&mut *trans)
        .await?;
        sqlx::query!(
            "DELETE FROM subscriptions WHERE user_id = ? AND device_id = ?",
            user.id,
            device_id
        )
        .execute(&mut *trans)
        .await?;
        sqlx::query!("DELETE FROM episode_actions WHERE device_id = ?", device_id)
            .execute(&mut *trans)
            .await?;

        trans.commit().await?;

        Ok(())
    }
}

pub struct Subscription {
    pub podcast: String,
    pub action: String,
    pub timestamp: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct ChangesRequest {
    pub add: Vec<String>,
    pub remove: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct ChangesResponse {
    pub add: Vec<String>,
    pub remove: Vec<String>,
    pub timestamp: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct ChangesQuery {
    pub since: i64,
}

impl Database {
    #[tracing::instrument(skip_all, err)]
    #[autometrics::autometrics]
    pub async fn add_subscription<'c, E: sqlx::Executor<'c, Database = sqlx::Sqlite>>(
        &self,
        conn: E,
        user: &User,
        device_id: i64,
        timestamp: i64,
        action: &str,
        podcast: &str,
    ) -> Result<()> {
        sqlx::query!("INSERT INTO subscriptions (user_id, device_id, podcast, action, timestamp) VALUES (?,?,?,?,?)", user.id, device_id, podcast, action, timestamp)
            .execute(conn)
            .await
            .map(|_| ())
            .map_err(Error::from)
    }

    #[tracing::instrument(skip_all, err)]
    #[autometrics::autometrics]
    pub async fn add_subscriptions(
        &self,
        user: &User,
        device_id: i64,
        subscriptions: ChangesRequest,
    ) -> Result<Vec<Vec<String>>> {
        let mut trans = self.pool.begin().await?;

        let timestamp = OffsetDateTime::now_utc().unix_timestamp();

        for added in &subscriptions.add {
            self.add_subscription(&mut *trans, user, device_id, timestamp, "subscribe", added)
                .await?;
        }

        for removed in &subscriptions.remove {
            self.add_subscription(
                &mut *trans,
                user,
                device_id,
                timestamp,
                "unsubscribe",
                removed,
            )
            .await?;
        }

        trans.commit().await?;

        Ok(vec![subscriptions.add, subscriptions.remove])
    }

    #[tracing::instrument(skip_all, err)]
    #[autometrics::autometrics]
    pub async fn subscription_history(
        &self,
        user: &User,
        device_id: i64,
        since: i64,
    ) -> Result<Vec<Subscription>> {
        sqlx::query_as!(Subscription, "SELECT podcast, action, timestamp FROM subscriptions WHERE user_id = ? AND device_id = ? AND timestamp > ?", user.id, device_id, since)
            .fetch_all(&self.pool)
            .await
            .map_err(Error::from)
    }
}
