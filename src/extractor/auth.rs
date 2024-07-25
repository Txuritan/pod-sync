use anyhow::Result;
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::IntoResponse,
};
use axum_extra::TypedHeader;
use headers::{authorization::Bearer, Authorization};
use time::OffsetDateTime;

use crate::database::User;

#[derive(Debug, thiserror::Error)]
pub enum SessionRejection {
    #[error("Missing authorization header")]
    MissingHeader,
    #[error("Unauthorized")]
    Unauthorized,
}

impl IntoResponse for SessionRejection {
    fn into_response(self) -> axum::response::Response {
        tracing::error!(err = %self, "failed to authenticate request");

        match self {
            SessionRejection::MissingHeader => StatusCode::UNAUTHORIZED,
            SessionRejection::Unauthorized => StatusCode::UNAUTHORIZED,
        }
        .into_response()
    }
}

pub struct Session {
    pub expires: OffsetDateTime,
    pub user: User,
}

impl Session {
    pub fn validate(&self) -> bool {
        let now = OffsetDateTime::now_utc();
        let expires = self.expires;

        expires < now
    }
}

#[async_trait::async_trait]
impl FromRequestParts<crate::Sync> for Session {
    type Rejection = SessionRejection;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &crate::Sync,
    ) -> Result<Self, Self::Rejection> {
        let header = TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
            .await
            .map_err(|_| SessionRejection::MissingHeader)?;

        let Some(session) = state
            .db
            .user_get_by_token(header.0.token())
            .await
            .map_err(|_| SessionRejection::Unauthorized)?
        else {
            return Err(SessionRejection::Unauthorized);
        };

        Ok(session)
    }
}
