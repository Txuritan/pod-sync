pub mod mutation;
pub mod query;

use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, sqlx::Type)]
#[sqlx(transparent)]
pub struct SubscriptionId(i64);

impl From<i64> for SubscriptionId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, sqlx::Type)]
#[sqlx(transparent)]
pub struct UserSubscriptionId(i64);

impl From<i64> for UserSubscriptionId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

pub struct RowSubscription {
    pub id: SubscriptionId,
    pub created: OffsetDateTime,
    pub updated: Option<OffsetDateTime>,
    pub deleted: Option<OffsetDateTime>,
}

pub struct RowUserSubscription {
    pub user_id: i64,
    pub subscription_id: SubscriptionId,
    pub created: OffsetDateTime,
    pub updated: Option<OffsetDateTime>,
    pub deleted: Option<OffsetDateTime>,
}

pub struct RowSubscriptionFeed {
    pub subscription_id: SubscriptionId,
    pub feed: String, // TODO: switch this to a Url
    pub created: OffsetDateTime,
    pub updated: Option<OffsetDateTime>,
    pub deleted: Option<OffsetDateTime>,
}

pub struct RowSubscriptionGuid {
    pub subscription_id: SubscriptionId,
    pub guid: Uuid,
    pub created: OffsetDateTime,
    pub updated: Option<OffsetDateTime>,
    pub deleted: Option<OffsetDateTime>,
}

pub struct WrapperId {
    pub id: SubscriptionId,
}
