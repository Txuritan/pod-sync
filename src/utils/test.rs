use std::fmt::Debug;

use axum::{
    body::Body,
    http::{header, HeaderValue, Method, Request, StatusCode, Uri},
    Router,
};
use http_body_util::BodyExt as _;
use mediatype::{
    names::{APPLICATION, CHARSET, JSON, XML},
    values::UTF_8,
    MediaTypeBuf,
};
use pretty_assertions::assert_eq;
use tower::ServiceExt as _;

use crate::database::Database;

#[derive(Clone, Copy)]
pub enum Format {
    Json,
    Xml,
}

pub struct TestBuilder<T>
where
    T: Debug + PartialEq + serde::de::DeserializeOwned,
{
    app: Router,
    method: Method,
    accepts: Format,
    authorization: bool,
    url: Uri,
    content: Format,
    body: Option<(Format, Body)>,
    status: StatusCode,
    expected: T,
}

impl<T> TestBuilder<T>
where
    T: Debug + PartialEq + serde::de::DeserializeOwned,
{
    pub fn new<U>(app: Router, url: U, expected: T) -> Self
    where
        U: TryInto<Uri>,
        <U as TryInto<Uri>>::Error: Debug,
    {
        Self {
            app,
            method: Method::GET,
            accepts: Format::Json,
            authorization: false,
            url: url.try_into().unwrap(),
            content: Format::Json,
            body: None,
            status: StatusCode::OK,
            expected,
        }
    }

    pub fn method(self, method: Method) -> Self {
        Self { method, ..self }
    }

    pub fn accepts(self, accepts: Format) -> Self {
        Self { accepts, ..self }
    }

    pub fn authorization(self, authorization: bool) -> Self {
        Self {
            authorization,
            ..self
        }
    }

    pub fn content(self, content: Format) -> Self {
        Self { content, ..self }
    }

    pub fn body(self, format: Format, body: Body) -> Self {
        Self {
            body: Some((format, body)),
            ..self
        }
    }

    pub fn status(self, status: StatusCode) -> Self {
        Self { status, ..self }
    }

    pub async fn run(self) {
        let Self {
            app,
            method,
            accepts,
            authorization,
            url,
            content,
            body,
            status,
            expected,
        } = self;

        let accept_type = match accepts {
            Format::Json => MediaTypeBuf::new(APPLICATION, JSON),
            Format::Xml => MediaTypeBuf::new(APPLICATION, XML),
        };

        let content_type = match content {
            Format::Json => MediaTypeBuf::new(APPLICATION, JSON),
            Format::Xml => MediaTypeBuf::from_parts(APPLICATION, XML, None, &[(CHARSET, UTF_8)]),
        };

        let builder = Request::builder().method(method);

        let builder = if authorization {
            builder.header(header::AUTHORIZATION, Database::test_token())
        } else {
            builder
        };

        let builder = builder.uri(url).header(
            header::ACCEPT,
            HeaderValue::from_str(&accept_type.to_string())
                .expect("Accept header value was not valid, this should not happen"),
        );

        let request = if let Some((format, body)) = body {
            let format_type = match format {
                Format::Json => MediaTypeBuf::new(APPLICATION, JSON),
                Format::Xml => MediaTypeBuf::new(APPLICATION, XML),
            };

            builder
                .header(
                    header::CONTENT_TYPE,
                    HeaderValue::from_str(&format_type.to_string())
                        .expect("Content-Type header value was not valid, this should not happen"),
                )
                .body(body)
        } else {
            builder.body(Body::empty())
        }
        .expect("Failed to build request");

        let response = app.oneshot(request).await.expect("Failed to run request");

        assert_eq!(
            response.status(),
            status,
            "Response status code did not match the expected value"
        );

        let header = response
            .headers()
            .get(header::CONTENT_TYPE)
            .expect("Missing content-type header");
        let header = String::from_utf8(header.as_bytes().to_vec())
            .expect("Header content-type did not return using UTF-8");
        let header = header
            .parse::<MediaTypeBuf>()
            .expect("Header content-type did not respond with a valid media type");

        assert_eq!(
            content_type, header,
            "Content-Type header did not match the expected valid"
        );

        let body = response
            .into_body()
            .collect()
            .await
            .expect("Failed to collect response body")
            .to_bytes();

        assert!(!body.is_empty(), "Response body should never be empty");

        let body = match content {
            Format::Json => {
                serde_json::from_slice(&body[..]).expect("Failed to deserialize response body")
            }
            Format::Xml => {
                let text = String::from_utf8(body.to_vec())
                    .expect("Failed to convert body into UTF-8 text");

                quick_xml::de::from_str(&text).expect("Failed to deserialize response body")
            }
        };

        assert_eq!(
            expected, body,
            "Response body did not match the expected value"
        );
    }
}
