use argon2::{
    password_hash::SaltString, Argon2, PasswordHash, PasswordHasher as _, PasswordVerifier as _,
};
use rand::rngs::OsRng;

use crate::{
    database::Database,
    error::{Error, Result},
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
}
