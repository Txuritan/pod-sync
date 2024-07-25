use time::OffsetDateTime;
use url::Url;
use uuid::Uuid;

use crate::{
    database::{user::User, Database},
    error::{Error, Result},
    models::subscriptions::{Subscription, Subscriptions},
};

#[derive(Clone, Copy, sqlx::Type)]
#[sqlx(transparent)]
pub struct SubscriptionId(i64);

impl From<i64> for SubscriptionId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, sqlx::Type)]
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

impl Database {
    async fn subscription_get_id_by_guid(&self, uuid: Uuid) -> Result<Option<RowSubscriptionGuid>> {
        sqlx::query_as!(
            RowSubscriptionGuid,
            r#"
                SELECT subscription_id, guid as "guid: Uuid", created, updated, deleted
                FROM subscription_guids
                WHERE guid = ?1
            "#,
            uuid,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::from)
    }

    // TODO: add since support so we dont have to process as much
    async fn subscription_get_feeds(&self, id: SubscriptionId) -> Result<Vec<RowSubscriptionFeed>> {
        sqlx::query_as!(
            RowSubscriptionFeed,
            r#"
                SELECT subscription_id, feed, created, updated, deleted
                FROM subscription_feeds
                WHERE subscription_id = ?1
            "#,
            id,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::from)
    }

    // TODO: add since support so we dont have to process as much
    async fn subscription_get_guids(&self, id: SubscriptionId) -> Result<Vec<RowSubscriptionGuid>> {
        sqlx::query_as!(
            RowSubscriptionGuid,
            r#"
                SELECT subscription_id, guid as "guid: Uuid", created, updated, deleted
                FROM subscription_guids
                WHERE subscription_id = ?1
            "#,
            id,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::from)
    }

    async fn subscriptions_fill_all(
        &self,
        user: &User,
        page: i64,
        per_page: i64,
        ids: Vec<WrapperId>,
    ) -> Result<Option<Subscriptions>> {
        let mut subscriptions = Vec::with_capacity(ids.len());

        for id in ids {
            let Some(subscription) = self.subscription_get_by_id(user, id.id).await? else {
                return Ok(None);
            };

            subscriptions.push(subscription);
        }

        Ok(Some(Subscriptions {
            total: subscriptions.len() as i64, // TODO: query total amount of subscriptions
            page,
            per_page,
            next: None,
            previous: None,
            subscriptions,
        }))
    }

    pub async fn subscriptions_get_all(
        &self,
        user: &User,
        page: Option<i64>,
        per_page: Option<i64>,
    ) -> Result<Option<Subscriptions>> {
        let mut page = page.unwrap_or(1);
        let mut per_page = per_page.unwrap_or(50);

        if page == 0 {
            page = 1;
        }
        if per_page == 0 {
            per_page = 50;
        }

        let offset = (per_page * page) - per_page;

        // TODO: look into https://gist.github.com/ssokolow/262503 for paging
        let ids = sqlx::query_as!(
            WrapperId,
            r#"
                SELECT s.id
                FROM user_subscriptions us
                LEFT JOIN subscriptions s ON us.subscription_id = s.id
                WHERE us.user_id = ?1
                ORDER BY us.created DESC
                LIMIT ?2
                OFFSET ?3
            "#,
            user.id,
            per_page,
            offset,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::from)?;

        self.subscriptions_fill_all(user, page, per_page, ids).await
    }

    pub async fn subscriptions_get_all_since(
        &self,
        user: &User,
        since: OffsetDateTime,
        page: Option<i64>,
        per_page: Option<i64>,
    ) -> Result<Option<Subscriptions>> {
        let mut page = page.unwrap_or(1);
        let mut per_page = per_page.unwrap_or(50);

        if page == 0 {
            page = 1;
        }
        if per_page == 0 {
            per_page = 50;
        }

        let offset = (per_page * page) - per_page;

        // TODO: look into https://gist.github.com/ssokolow/262503 for paging
        let ids = sqlx::query_as!(
            WrapperId,
            r#"
                SELECT s.id
                FROM user_subscriptions us
                LEFT JOIN subscriptions s ON us.subscription_id = s.id
                WHERE us.user_id = ?1 AND (us.created < ?4 OR us.updated < ?4 OR us.deleted < ?4)
                ORDER BY us.created DESC
                LIMIT ?2
                OFFSET ?3
            "#,
            user.id,
            per_page,
            offset,
            since,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::from)?;

        self.subscriptions_fill_all(user, page, per_page, ids).await
    }

    pub async fn subscription_get_by_id(
        &self,
        user: &User,
        id: SubscriptionId,
    ) -> Result<Option<Subscription>> {
        let subscription = sqlx::query_as!(
            RowUserSubscription,
            r#"
                SELECT us.user_id, us.subscription_id, us.created, us.updated, us.deleted
                FROM user_subscriptions us
                LEFT JOIN subscriptions s ON us.subscription_id = s.id
                WHERE us.user_id = ?1 AND us.subscription_id = ?2
            "#,
            user.id,
            id,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::from)?;

        let Some(subscription) = subscription else {
            return Ok(None);
        };

        let mut feeds = self.subscription_get_feeds(id).await?;
        let guids = self.subscription_get_guids(id).await?;

        if feeds.is_empty() {
            return Ok(None);
        }
        if guids.is_empty() {
            return Ok(None);
        }

        let feed_row = feeds
            .pop()
            .expect("feeds was empty, this should not be the case");
        let feed_url = Url::parse(&feed_row.feed)?;

        let guid_row = guids
            .first()
            .expect("guids was empty, this should be the case");
        let guid = guid_row.guid;

        let new_guid_row = guids.last();
        let new_guid = new_guid_row.map(|g| g.guid);

        let guid_changed = new_guid_row.and_then(|g| g.updated);

        Ok(Some(Subscription {
            feed_url,
            guid,
            is_subscribed: subscription.deleted.is_none(),
            subscription_changed: subscription.updated,
            new_guid,
            guid_changed,
            deleted: subscription.deleted,
        }))
    }

    pub async fn subscription_get_by_guid(
        &self,
        user: &User,
        uuid: Uuid,
    ) -> Result<Option<Subscription>> {
        let Some(row) = self.subscription_get_id_by_guid(uuid).await? else {
            return Ok(None);
        };

        self.subscription_get_by_id(user, row.subscription_id).await
    }
}
