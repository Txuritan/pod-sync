use anyhow::Context as _;
use time::OffsetDateTime;
use url::Url;
use uuid::Uuid;

use crate::{
    database::{
        subscription::{
            RowSubscriptionFeed, RowSubscriptionGuid, RowUserSubscription, SubscriptionId,
            WrapperId,
        },
        user::User,
        Database,
    },
    models::subscriptions::{Subscription, Subscriptions},
};

impl Database {
    #[tracing::instrument(skip_all, err)]
    #[autometrics::autometrics]
    pub async fn subscription_get_id_by_guid(
        &self,
        uuid: Uuid,
    ) -> anyhow::Result<Option<RowSubscriptionGuid>> {
        sqlx::query_as!(
            RowSubscriptionGuid,
            r#"--sql
                SELECT
                    subscription_id, guid as "guid: Uuid", created, updated, deleted
                FROM
                    subscription_guids
                WHERE
                    guid = ?1
            "#,
            uuid,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(anyhow::Error::from)
        .context("Failed to run query: get subscriptions by guid")
    }

    // TODO: add since support so we don't have to process as much
    #[tracing::instrument(skip_all, err)]
    #[autometrics::autometrics]
    async fn subscription_get_feeds(
        &self,
        id: SubscriptionId,
    ) -> anyhow::Result<Vec<RowSubscriptionFeed>> {
        sqlx::query_as!(
            RowSubscriptionFeed,
            r#"--sql
                SELECT
                    subscription_id, feed, created, updated, deleted
                FROM
                    subscription_feeds
                WHERE
                    subscription_id = ?1
                ORDER BY created ASC
            "#,
            id,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(anyhow::Error::from)
        .context("Failed to run query: get subscriptions feeds")
    }

    // TODO: add since support so we don't have to process as much
    #[tracing::instrument(skip_all, err)]
    #[autometrics::autometrics]
    async fn subscription_get_guids(
        &self,
        id: SubscriptionId,
    ) -> anyhow::Result<Vec<RowSubscriptionGuid>> {
        sqlx::query_as!(
            RowSubscriptionGuid,
            r#"--sql
                SELECT
                    subscription_id, guid as "guid: Uuid", created, updated, deleted
                FROM
                    subscription_guids
                WHERE
                    subscription_id = ?1
                ORDER BY created ASC
            "#,
            id,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(anyhow::Error::from)
        .context("Failed to run query: get subscriptions guids")
    }

    #[tracing::instrument(skip_all, err)]
    #[autometrics::autometrics]
    async fn subscriptions_fill_all(
        &self,
        user: &User,
        page: i64,
        per_page: i64,
        ids: Vec<WrapperId>,
    ) -> anyhow::Result<Option<Subscriptions>> {
        let mut subscriptions = Vec::with_capacity(ids.len());

        for id in ids {
            let subscription = self
                .subscription_get_by_id(user, id.id)
                .await
                .context("Failed to fill out subscription")?;
            let Some(subscription) = subscription else {
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

    #[tracing::instrument(skip_all, err)]
    #[autometrics::autometrics]
    pub async fn subscriptions_get_all(
        &self,
        user: &User,
        page: Option<i64>,
        per_page: Option<i64>,
    ) -> anyhow::Result<Option<Subscriptions>> {
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
            r#"--sql
                SELECT
                    s.id
                FROM
                    user_subscriptions us
                LEFT JOIN subscriptions s ON us.subscription_id = s.id
                WHERE
                    us.user_id = ?1
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
        .map_err(anyhow::Error::from)
        .context("Failed to run query: get user subscriptions")?;

        self.subscriptions_fill_all(user, page, per_page, ids).await
    }

    #[tracing::instrument(skip_all, err)]
    #[autometrics::autometrics]
    pub async fn subscriptions_get_all_since(
        &self,
        user: &User,
        since: OffsetDateTime,
        page: Option<i64>,
        per_page: Option<i64>,
    ) -> anyhow::Result<Option<Subscriptions>> {
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
            r#"--sql
                SELECT
                    s.id
                FROM
                    user_subscriptions us
                LEFT JOIN subscriptions s ON us.subscription_id = s.id
                WHERE
                    us.user_id = ?1 AND (us.created < ?4 OR us.updated < ?4 OR us.deleted < ?4)
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
        .map_err(anyhow::Error::from)
        .context("Failed to run query: get user subscriptions since date")?;

        self.subscriptions_fill_all(user, page, per_page, ids).await
    }

    #[tracing::instrument(skip_all, err)]
    #[autometrics::autometrics]
    pub async fn subscription_get_by_id(
        &self,
        user: &User,
        id: SubscriptionId,
    ) -> anyhow::Result<Option<Subscription>> {
        let subscription = sqlx::query_as!(
            RowUserSubscription,
            r#"--sql
                SELECT
                    us.user_id, us.subscription_id, us.created, us.updated, us.deleted
                FROM
                    user_subscriptions us
                LEFT JOIN subscriptions s ON us.subscription_id = s.id
                WHERE
                    us.user_id = ?1 AND us.subscription_id = ?2
            "#,
            user.id,
            id,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(anyhow::Error::from)
        .context("Failed to run query: get user subscriptions by id")?;

        let Some(subscription) = subscription else {
            tracing::debug!("User subscriptions query returned None");

            return Ok(None);
        };

        let mut feeds = self
            .subscription_get_feeds(id)
            .await
            .context("Failed get subscription feeds")?;
        let guids = self
            .subscription_get_guids(id)
            .await
            .context("Failed get subscription guids")?;

        if feeds.is_empty() {
            tracing::debug!("Subscription feeds query returned None");

            return Ok(None);
        }
        if guids.is_empty() {
            tracing::debug!("Subscription guids query returned None");

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
        let mut new_guid = new_guid_row.map(|g| g.guid);
        if new_guid == Some(guid) {
            new_guid = None;
        }

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

    #[tracing::instrument(skip_all, err)]
    #[autometrics::autometrics]
    pub async fn subscription_get_by_guid(
        &self,
        user: &User,
        uuid: Uuid,
    ) -> anyhow::Result<Option<Subscription>> {
        let row = self
            .subscription_get_id_by_guid(uuid)
            .await
            .context("Failed to get subscription id from its guid")?;
        let Some(row) = row else {
            return Ok(None);
        };

        self.subscription_get_by_id(user, row.subscription_id)
            .await
            .context("Failed to get subscription by its id")
    }
}
