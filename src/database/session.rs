use data_encoding::BASE64;
use rand::{rngs::OsRng, RngCore as _};
use time::OffsetDateTime;

use crate::{
    database::{user::User, Database},
    extractor::auth::Session,
};

#[derive(sqlx::FromRow)]
pub struct OptionalSession {
    pub id: Option<i64>,
    pub username: Option<String>,
    pub email: Option<String>,
    pub password_hash: Option<String>,
    pub expires: Option<OffsetDateTime>,
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

impl Database {
    #[tracing::instrument(skip_all, err)]
    #[autometrics::autometrics]
    pub async fn session_crate(&self, user: &User) -> anyhow::Result<(String, OffsetDateTime)> {
        let now = OffsetDateTime::now_utc();
        let expires = now + time::Duration::days(7 * 3);

        let mut bytes = [0; 64];
        OsRng.fill_bytes(&mut bytes);
        let token = BASE64.encode(&bytes);

        sqlx::query!(
            r#"
                INSERT INTO
                    user_sessions (user_id, token, expires )
                VALUES
                    ( ?, ?, ? )
            "#,
            user.id,
            token,
            expires
        )
        .execute(&self.pool)
        .await?;

        Ok((token, expires))
    }

    #[tracing::instrument(skip_all, err)]
    #[autometrics::autometrics]
    pub async fn session_get_by_token(&self, token: &str) -> anyhow::Result<Option<Session>> {
        sqlx::query_as!(
            OptionalSession,
            r#"
                SELECT
                    u.id, u.username, u.email, u.password_hash, us.expires
                FROM
                    user_sessions us
                LEFT JOIN users u ON us.user_id = u.id
                WHERE
                    us.token = ?
                LIMIT 1
            "#,
            token
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(anyhow::Error::from)
        .map(|ok| ok.and_then(OptionalSession::into_session))
    }
}
