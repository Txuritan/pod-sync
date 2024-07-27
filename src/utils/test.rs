use std::fmt::Debug;

use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode, Uri},
    Router,
};
use http_body_util::BodyExt as _;
use pretty_assertions::assert_eq;
use tower::ServiceExt as _;

use crate::database::Database;

pub struct TestBuilder<T>
where
    T: Debug + PartialEq + serde::de::DeserializeOwned,
{
    app: Router,
    method: Method,
    authorization: bool,
    url: Uri,
    body: Option<Body>,
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
            authorization: false,
            url: url.try_into().unwrap(),
            body: None,
            status: StatusCode::OK,
            expected,
        }
    }

    pub fn method(self, method: Method) -> Self {
        Self { method, ..self }
    }

    pub fn authorization(self, authorization: bool) -> Self {
        Self {
            authorization,
            ..self
        }
    }

    pub fn body(self, body: Body) -> Self {
        Self {
            body: Some(body),
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
            authorization,
            url,
            body,
            status,
            expected,
        } = self;

        let builder = Request::builder().method(method);

        let builder = if authorization {
            builder.header(header::AUTHORIZATION, Database::test_token())
        } else {
            builder
        };

        let builder = builder
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .uri(url);

        let request = if let Some(body) = body {
            builder.body(body)
        } else {
            builder.body(Body::empty())
        }
        .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), status);

        let body = response.into_body().collect().await.unwrap().to_bytes();

        assert!(!body.is_empty());

        let body: T = serde_json::from_slice(&body[..]).unwrap();

        assert_eq!(expected, body);
    }
}
