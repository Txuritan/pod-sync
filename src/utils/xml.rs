use std::ops::{Deref, DerefMut};

use axum::{
    body::{Body, Bytes},
    extract::{rejection::BytesRejection, FromRequest, Request},
    http::{
        header::{self, HeaderValue},
        StatusCode,
    },
    response::{IntoResponse, Response},
};
use serde::{de::DeserializeOwned, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum XmlRejection {
    #[error("Failed to parse the request body as XML")]
    InvalidXMLBody(#[from] quick_xml::DeError),
    #[error("Expected request with `Content-Type: application/xml`")]
    MissingXMLContentType,
    #[error("{0}")]
    BytesRejection(#[from] BytesRejection),
}

impl IntoResponse for XmlRejection {
    fn into_response(self) -> Response {
        match self {
            e @ XmlRejection::InvalidXMLBody(_) => {
                let mut res = Response::new(Body::from(format!("{}", e)));
                *res.status_mut() = StatusCode::UNPROCESSABLE_ENTITY;
                res
            }
            e @ XmlRejection::MissingXMLContentType => {
                let mut res = Response::new(Body::from(format!("{}", e)));
                *res.status_mut() = StatusCode::UNSUPPORTED_MEDIA_TYPE;
                res
            }
            XmlRejection::BytesRejection(e) => e.into_response(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Xml<T>(pub T);

#[async_trait::async_trait]
impl<T, S> FromRequest<S> for Xml<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = XmlRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        if xml_content_type(&req) {
            let bytes = Bytes::from_request(req, state).await?;

            let value = quick_xml::de::from_reader(&*bytes)?;

            Ok(Self(value))
        } else {
            Err(XmlRejection::MissingXMLContentType)
        }
    }
}

fn xml_content_type(req: &Request) -> bool {
    let content_type = if let Some(content_type) = req.headers().get(header::CONTENT_TYPE) {
        content_type
    } else {
        return false;
    };

    let content_type = if let Ok(content_type) = content_type.to_str() {
        content_type
    } else {
        return false;
    };

    let mime = if let Ok(mime) = content_type.parse::<mime::Mime>() {
        mime
    } else {
        return false;
    };

    let is_xml_content_type = (mime.type_() == "application" || mime.type_() == "text")
        && (mime.subtype() == "xml" || mime.suffix().map_or(false, |name| name == "xml"));

    is_xml_content_type
}

impl<T> Deref for Xml<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Xml<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("text/plain; charset=utf-8"),
                )],
                err.to_string(),
            )
                .into_response();
        }

        (
            [(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/xml; charset=utf-8"),
            )],
            buf,
        )
            .into_response()
    }
}
