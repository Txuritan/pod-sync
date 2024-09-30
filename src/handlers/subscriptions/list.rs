use axum::extract::{Query, State};
use axum_extra::either::Either3;
use time::OffsetDateTime;

use crate::{
    extractor::auth::Session,
    models::{subscriptions::Subscriptions, InternalError, Unauthorized},
    SyncState,
};

#[derive(serde::Deserialize)]
pub struct ListParams {
    pub since: Option<OffsetDateTime>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[tracing::instrument(skip_all)]
#[autometrics::autometrics]
pub async fn list(
    State(sync): State<SyncState>,
    session: Option<Session>,
    Query(params): Query<ListParams>,
) -> Either3<Subscriptions, Unauthorized, InternalError> {
    let Some(session) = session else {
        tracing::info!("no session");
        return Either3::E2(Unauthorized);
    };
    if !session.validate() {
        tracing::info!("session invalid");
        return Either3::E2(Unauthorized);
    }

    let ListParams {
        since,
        page,
        per_page,
    } = params;

    let subscriptions = match since {
        Some(since) => {
            sync.db
                .subscriptions_get_all_since(&session.user, since, page, per_page)
                .await
        }
        None => {
            sync.db
                .subscriptions_get_all(&session.user, page, per_page)
                .await
        }
    };

    match subscriptions {
        Ok(Some(subscriptions)) => Either3::E1(subscriptions),
        Ok(None) => Either3::E1(Subscriptions::empty()),
        Err(err) => {
            tracing::error!(err = ?err, "Failed to retrieve user subscriptions");

            Either3::E3(InternalError)
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::{
        http::{Method, StatusCode},
        routing::get,
        Router,
    };
    use url::Url;

    use crate::{
        database::Database,
        handlers::test_app,
        models::{
            subscriptions::{Subscription, Subscriptions},
            ApiError,
        },
        utils::test::TestBuilder,
    };

    async fn setup_app(pool: sqlx::SqlitePool) -> Router {
        test_app(pool, |router| {
            router.route("/v1/subscriptions", get(super::list))
        })
        .await
        .expect("failed to setup app")
    }

    #[sqlx::test(fixtures("../../../fixtures/dummy.sql"))]
    async fn ok(pool: sqlx::SqlitePool) {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();

        let app = setup_app(pool).await;
        let url = "/v1/subscriptions";
        let expected = Subscriptions {
            total: 3,
            page: 1,
            per_page: 50,
            next: None,
            previous: None,
            subscriptions: vec![
                Subscription {
                    feed_url: Url::parse(Database::SUBSCRIPTION_3_FEED).unwrap(),
                    guid: Database::SUBSCRIPTION_3_GUID_OLD,
                    is_subscribed: true,
                    subscription_changed: None,
                    new_guid: Some(Database::SUBSCRIPTION_3_GUID_NEW),
                    guid_changed: None,
                    deleted: None,
                },
                Subscription {
                    feed_url: Url::parse(Database::SUBSCRIPTION_1_FEED).unwrap(),
                    guid: Database::SUBSCRIPTION_1_GUID,
                    is_subscribed: true,
                    subscription_changed: None,
                    new_guid: None,
                    guid_changed: None,
                    deleted: None,
                },
                Subscription {
                    feed_url: Url::parse(Database::SUBSCRIPTION_2_FEED_NEW).unwrap(),
                    guid: Database::SUBSCRIPTION_2_GUID,
                    is_subscribed: true,
                    subscription_changed: None,
                    new_guid: None,
                    guid_changed: None,
                    deleted: None,
                },
            ],
        };

        TestBuilder::new(app, url, expected)
            .method(Method::GET)
            .authorization(true)
            .status(StatusCode::OK)
            .run()
            .await;
    }

    #[sqlx::test(fixtures("../../../fixtures/dummy.sql"))]
    async fn unauthorized(pool: sqlx::SqlitePool) {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();

        let app = setup_app(pool).await;
        let url = "/v1/subscriptions";
        let expected = ApiError::unauthorized();

        TestBuilder::new(app, url, expected)
            .method(Method::GET)
            .status(StatusCode::UNAUTHORIZED)
            .run()
            .await;
    }
}
