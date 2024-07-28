use axum::{
    extract::{FromRequest, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::TypedHeader;
use bytes::{BufMut, Bytes, BytesMut};
use serde::{de::DeserializeOwned, Serialize};

use crate::utils::content_type::ContentType;

static BAD_REQUEST: &str = r#"{"code":400,"message":"Input failed to decode"}"#;
static INTERNAL_ERROR: &str = r#"{"code":500,"message":"Internal error"}"#;

pub struct JsonRejection;

impl IntoResponse for JsonRejection {
    fn into_response(self) -> Response {
        (
            StatusCode::BAD_REQUEST,
            TypedHeader(ContentType::json()),
            BAD_REQUEST,
        )
            .into_response()
    }
}

#[derive(Debug, Clone, Copy, Default)]
#[must_use]
pub struct Json<T>(pub T);

#[async_trait::async_trait]
impl<T, S> FromRequest<S> for Json<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = JsonRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let bytes = match Bytes::from_request(req, state).await {
            Ok(bytes) => bytes,
            Err(err) => {
                tracing::error!(err = %err, "Failed to read body");

                return Err(JsonRejection);
            }
        };

        match serde_json::from_slice(&bytes) {
            Ok(value) => Ok(Self(value)),
            Err(err) => {
                tracing::error!(err = %err, "Failed to deserialize JSON body");

                Err(JsonRejection)
            }
        }
    }
}

impl<T> From<T> for Json<T> {
    fn from(inner: T) -> Self {
        Self(inner)
    }
}

impl<T> IntoResponse for Json<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        let mut buf = BytesMut::with_capacity(128).writer();

        if let Err(err) = serde_json::to_writer(&mut buf, &self.0) {
            tracing::error!(err = %err, "Failed to encode JSON response");

            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                TypedHeader(ContentType::json()),
                INTERNAL_ERROR,
            )
                .into_response();
        }

        (TypedHeader(ContentType::json()), buf.into_inner().freeze()).into_response()
    }
}
