use anyhow::Result;
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::IntoResponse,
};
use axum_extra::{
    extract::{cookie::Key, PrivateCookieJar},
    TypedHeader,
};
use headers::{authorization::Basic, Authorization};
use time::OffsetDateTime;

use crate::{database::User, error::Error};

pub static SESSION: &str = "pod-sync-session";

pub struct Session {
    pub expires: i64,
    pub user: User,
}

impl Session {
    pub fn validate(&self) -> bool {
        let now = OffsetDateTime::now_utc();
        let Ok(expires) = OffsetDateTime::from_unix_timestamp(self.expires) else {
            return false;
        };

        expires < now
    }
}

#[tracing::instrument(skip_all, err)]
async fn from_request_parts(
    parts: &mut Parts,
    state: &crate::Sync,
) -> Result<Option<Session>, AuthRejection> {
    let header = Option::<TypedHeader<Authorization<Basic>>>::from_request_parts(parts, state)
        .await
        .ok()
        .flatten();
    if let Some(auth) = header {
        let username = auth.username();
        let password = auth.password();

        let Some(user) = state
            .db
            .user_get_by_username(username)
            .await
            .map_err(|_| AuthRejection::Unauthorized)?
        else {
            return Err(AuthRejection::Unauthorized);
        };

        if user.verify(password) {
            return Ok(Some(Session { expires: -1, user }));
        }
    }

    let jar = PrivateCookieJar::<Key>::from_request_parts(parts, state)
        .await
        .expect("Infallible type errored");

    let Some(cookie) = jar.get(SESSION) else {
        return Ok(None);
    };

    let token = cookie.value();

    let session = state
        .db
        .user_get_session(token)
        .await
        .map_err(AuthRejection::InvalidSession)?
        .ok_or(AuthRejection::Unauthorized)?;

    if !session.validate() {
        return Err(AuthRejection::Unauthorized);
    }

    Ok(Some(session))
}

pub struct RequireAuthentication(pub Session);

#[async_trait::async_trait]
impl FromRequestParts<crate::Sync> for RequireAuthentication {
    type Rejection = AuthRejection;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &crate::Sync,
    ) -> Result<Self, Self::Rejection> {
        let session = from_request_parts(parts, state)
            .await?
            .ok_or(AuthRejection::MissingSessionCookie)?;

        Ok(Self(session))
    }
}

pub struct MaybeAuthenticated(pub Option<Session>);

#[async_trait::async_trait]
impl FromRequestParts<crate::Sync> for MaybeAuthenticated {
    type Rejection = AuthRejection;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &crate::Sync,
    ) -> Result<Self, Self::Rejection> {
        let session = from_request_parts(parts, state).await?;

        Ok(Self(session))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthRejection {
    #[error("Missing session cookie")]
    MissingSessionCookie,
    #[error("Invalid session")]
    InvalidSession(Error),
    #[error("Unauthorized")]
    Unauthorized,
}

impl IntoResponse for AuthRejection {
    fn into_response(self) -> axum::response::Response {
        tracing::error!(err = %self, "failed to authenticate request");

        match self {
            AuthRejection::MissingSessionCookie => StatusCode::BAD_REQUEST,
            AuthRejection::InvalidSession(_) => StatusCode::BAD_REQUEST,
            AuthRejection::Unauthorized => StatusCode::UNAUTHORIZED,
        }
        .into_response()
    }
}
