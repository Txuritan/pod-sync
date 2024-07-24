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
        let pool = sqlx::SqlitePool::connect("pod-sync.db").await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> sqlx::SqlitePool {
        self.pool.clone()
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
