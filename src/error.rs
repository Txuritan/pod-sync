use std::fmt;

use anyhow::Error as Inner;
use axum::response::{IntoResponse, Response};

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub struct Error(Inner);

impl fmt::Debug for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<Inner> for Error {
    #[inline]
    fn from(value: Inner) -> Self {
        Self(value)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        tracing::error!(err = %self, "failed to handle request");

        (axum::http::StatusCode::INTERNAL_SERVER_ERROR).into_response()
    }
}

macro_rules! impl_from {
    ($( $ty:path ),* $(,)?) => {
        $(
            impl From<$ty> for Error {
                #[inline]
                fn from(value: $ty) -> Self {
                    Self(Inner::from(value))
                }
            }
        )*
    };
}

impl_from![
    argon2::password_hash::Error,
    askama::Error,
    data_encoding::DecodeError,
    metrics_exporter_prometheus::BuildError,
    sqlx::Error,
    sqlx::migrate::MigrateError,
    std::io::Error,
    toml::de::Error,
    toml::ser::Error,
    url::ParseError,
];
