use time::OffsetDateTime;
use url::Url;
use uuid::Uuid;

#[derive(serde::Serialize)]
pub struct Subscription {
    pub feed_url: Url,
    pub guid: Uuid,
    pub is_subscribed: bool,
    pub subscription_changed: Option<OffsetDateTime>,
    pub new_guid: Option<Uuid>,
    pub guid_changed: Option<OffsetDateTime>,
    pub deleted: Option<OffsetDateTime>,
}

#[derive(serde::Serialize)]
pub struct Subscriptions {
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub subscriptions: Vec<Subscription>,
}

#[derive(serde::Serialize)]
pub struct NewSubscription {
    pub feed_url: Url,
    pub guid: Uuid,
    pub is_subscribed: bool,
    pub subscription_changed: OffsetDateTime,
}

#[derive(serde::Serialize)]
pub struct FailedSubscription {
    pub feed_url: Url,
    pub message: String,
}

#[derive(serde::Serialize)]
pub struct NewSubscriptions {
    pub success: Vec<NewSubscription>,
    pub failure: Vec<FailedSubscription>,
}

#[derive(serde::Serialize)]
pub struct SubscriptionUpdate {
    pub new_feed_url: Url,
    pub guid: Uuid,
    pub is_subscribed: bool,
}

pub type FeedArray = Vec<Feed>;

#[derive(serde::Deserialize)]
pub struct Feed {
    pub feed_url: Url,
    pub guid: Option<Uuid>,
}

#[derive(serde::Serialize)]
pub struct DeletionReceived {
    pub deletion_id: i64,
    pub message: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum DeletionStatus {
    Success,
    Pending,
    Failure,
}

#[derive(serde::Serialize)]
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
