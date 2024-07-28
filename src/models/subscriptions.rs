use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use time::OffsetDateTime;
use url::Url;
use uuid::Uuid;

use crate::utils::json::Json;

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Subscription {
    pub feed_url: Url,
    pub guid: Uuid,
    pub is_subscribed: bool,
    pub subscription_changed: Option<OffsetDateTime>,
    pub new_guid: Option<Uuid>,
    pub guid_changed: Option<OffsetDateTime>,
    pub deleted: Option<OffsetDateTime>,
}

impl IntoResponse for Subscription {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Subscriptions {
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub next: Option<Url>,
    pub previous: Option<Url>,
    pub subscriptions: Vec<Subscription>,
}

impl Subscriptions {
    pub fn empty() -> Self {
        Self {
            total: 0,
            page: 1,
            per_page: 0,
            next: None,
            previous: None,
            subscriptions: vec![],
        }
    }
}

impl IntoResponse for Subscriptions {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

#[derive(Debug, serde::Serialize)]
pub struct NewSubscription {
    pub feed_url: Url,
    pub guid: Uuid,
    pub is_subscribed: bool,
    pub subscription_changed: OffsetDateTime,
}

#[derive(Debug, serde::Serialize)]
pub struct FailedSubscription {
    pub feed_url: Url,
    pub message: String,
}

#[derive(Debug, serde::Serialize)]
pub struct NewSubscriptions {
    pub success: Vec<NewSubscription>,
    pub failure: Vec<FailedSubscription>,
}

impl IntoResponse for NewSubscriptions {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

#[derive(Debug, serde::Serialize)]
pub struct SubscriptionUpdate {
    pub new_feed_url: Url,
    pub guid: Uuid,
    pub is_subscribed: bool,
}

impl IntoResponse for SubscriptionUpdate {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct AddSubscriptions {
    pub subscriptions: Vec<Feed>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Feed {
    pub feed_url: String,
    pub guid: Option<Uuid>,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct DeletionReceived {
    pub deletion_id: i64,
    pub message: String,
}

impl DeletionReceived {
    pub fn new(deletion_id: i64) -> Self {
        Self {
            deletion_id,
            message: "Deletion request was received and will be processed".to_string(),
        }
    }
}

impl IntoResponse for DeletionReceived {
    fn into_response(self) -> Response {
        (StatusCode::CREATED, Json(self)).into_response()
    }
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, sqlx::Type)]
#[serde(rename_all = "UPPERCASE")]
#[sqlx(type_name = "deletion_status")]
#[sqlx(rename_all = "lowercase")]
pub enum DeletionStatus {
    Success,
    Pending,
    Failure,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Deletion {
    pub deletion_id: i64,
    pub status: DeletionStatus,
    pub message: String,
}

impl Deletion {
    pub fn success(deletion_id: i64) -> Self {
        Self {
            deletion_id,
            status: DeletionStatus::Success,
            message: "Subscription deleted successfully".to_string(),
        }
    }

    pub fn pending(deletion_id: i64) -> Self {
        Self {
            deletion_id,
            status: DeletionStatus::Pending,
            message: "Deletion is pending".to_string(),
        }
    }

    pub fn failure(deletion_id: i64) -> Self {
        Self {
            deletion_id,
            status: DeletionStatus::Success,
            message: "The deletion process encountered an error and was rolled back".to_string(),
        }
    }
}

impl IntoResponse for Deletion {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}
