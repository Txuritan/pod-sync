use axum::{
    extract::{FromRequest, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::TypedHeader;
use bytes::Bytes;
use serde::{de::DeserializeOwned, Serialize};

use crate::utils::content_type::ContentType;

static BAD_REQUEST: &str = r#"<?xml version="1.0" encoding="UTF-8" ?><error><code>400</code><message>Input failed to decode</message></error>"#;
static INTERNAL_ERROR: &str = r#"<?xml version="1.0" encoding="UTF-8" ?><error><code>500</code><message>Internal error</message></error>"#;

pub struct XmlRejection;

impl IntoResponse for XmlRejection {
    fn into_response(self) -> Response {
        (
            StatusCode::BAD_REQUEST,
            TypedHeader(ContentType::xml()),
            BAD_REQUEST,
        )
            .into_response()
    }
}

#[derive(Debug, Clone, Copy, Default)]
#[must_use]
pub struct Xml<T>(pub T);

#[async_trait::async_trait]
impl<T, S> FromRequest<S> for Xml<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = XmlRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let bytes = match Bytes::from_request(req, state).await {
            Ok(bytes) => bytes,
            Err(err) => {
                tracing::error!(err = %err, "Failed to read body");

                return Err(XmlRejection);
            }
        };

        match quick_xml::de::from_reader(&*bytes) {
            Ok(value) => Ok(Self(value)),
            Err(err) => {
                tracing::error!(err = %err, "Failed to deserialize XML body");

                Err(XmlRejection)
            }
        }
    }
}

impl<T> From<T> for Xml<T> {
    fn from(inner: T) -> Self {
        Self(inner)
    }
}

impl<T> IntoResponse for Xml<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        let mut buf = String::with_capacity(128);

        if let Err(err) = quick_xml::se::to_writer(&mut buf, &self.0) {
            tracing::error!(err = %err, "Failed to encode XML response");

            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                TypedHeader(ContentType::xml()),
                INTERNAL_ERROR,
            )
                .into_response();
        }

        (TypedHeader(ContentType::xml()), buf).into_response()
    }
}
