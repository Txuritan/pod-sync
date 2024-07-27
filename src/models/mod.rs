pub mod subscriptions;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ApiError {
    pub code: i64, // TODO: schema says string, examples say integer...
    pub message: String,
}

impl ApiError {
    pub fn unauthorized() -> Self {
        Self {
            code: 401,
            message: "User not authorized".to_string(),
        }
    }

    pub fn not_found() -> Self {
        Self {
            code: 404,
            message: "Resource not found".to_string(),
        }
    }

    pub fn validation() -> Self {
        Self {
            code: 405,
            message: "Input could not be validated".to_string(),
        }
    }

    pub fn gone() -> Self {
        Self {
            code: 410,
            message: "Subscription has been deleted".to_string(),
        }
    }
}

pub struct Unauthorized;

impl IntoResponse for Unauthorized {
    fn into_response(self) -> Response {
        (StatusCode::UNAUTHORIZED, Json(ApiError::unauthorized())).into_response()
    }
}

pub struct NotFound;

impl IntoResponse for NotFound {
    fn into_response(self) -> Response {
        (StatusCode::NOT_FOUND, Json(ApiError::not_found())).into_response()
    }
}

pub struct Validation;

impl IntoResponse for Validation {
    fn into_response(self) -> Response {
        // TODO: they use METHOD_NOT_ALLOWED here for some reason
        (StatusCode::BAD_REQUEST, Json(ApiError::validation())).into_response()
    }
}

pub struct Gone;

impl IntoResponse for Gone {
    fn into_response(self) -> Response {
        (StatusCode::GONE, Json(ApiError::gone())).into_response()
    }
}
