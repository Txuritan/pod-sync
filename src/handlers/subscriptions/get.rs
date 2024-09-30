use axum::extract::{Path, State};
use axum_extra::either::Either6;
use uuid::Uuid;

use crate::{
    extractor::auth::Session,
    models::{
        subscriptions::Subscription, Gone, InternalError, NotFound, Unauthorized, Validation,
    },
    SyncState,
};

#[tracing::instrument(skip_all)]
#[autometrics::autometrics]
pub async fn get(
    State(sync): State<SyncState>,
    session: Option<Session>,
    Path(guid): Path<Uuid>,
) -> Either6<Subscription, Unauthorized, NotFound, Validation, Gone, InternalError> {
    let Some(session) = session else {
        return Either6::E2(Unauthorized);
    };
    if !session.validate() {
        return Either6::E2(Unauthorized);
    }

    let subscription = match sync.db.subscription_get_by_guid(&session.user, guid).await {
        Ok(Some(subscription)) => subscription,
        Ok(None) => return Either6::E3(NotFound),
        Err(err) => {
            tracing::error!(err = ?err, "Failed to retrieve user subscription");

            return Either6::E6(InternalError);
        }
    };

    if subscription.deleted.is_some() {
        return Either6::E5(Gone);
    }

    Either6::E1(subscription)
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
        models::{subscriptions::Subscription, ApiError},
        utils::test::TestBuilder,
    };

    async fn setup_app(pool: sqlx::SqlitePool) -> Router {
        test_app(pool, |router| {
            router.route("/v1/subscriptions/:guid", get(super::get))
        })
        .await
        .expect("failed to setup app")
    }

    #[sqlx::test(fixtures("../../../fixtures/dummy.sql"))]
    async fn ok(pool: sqlx::SqlitePool) {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();

        let app = setup_app(pool).await;
        let url = format!("/v1/subscriptions/{}", Database::SUBSCRIPTION_1_GUID);
        let expected = Subscription {
            feed_url: Url::parse(Database::SUBSCRIPTION_1_FEED).unwrap(),
            guid: Database::SUBSCRIPTION_1_GUID,
            is_subscribed: true,
            subscription_changed: None,
            new_guid: None,
            guid_changed: None,
            deleted: None,
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
        let url = format!("/v1/subscriptions/{}", Database::SUBSCRIPTION_1_GUID);
        let expected = ApiError::unauthorized();

        TestBuilder::new(app, url, expected)
            .method(Method::GET)
            .status(StatusCode::UNAUTHORIZED)
            .run()
            .await;
    }

    // TODO: validation test

    #[sqlx::test(fixtures("../../../fixtures/dummy.sql"))]
    async fn not_found(pool: sqlx::SqlitePool) {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();

        let app = setup_app(pool).await;
        let url = format!("/v1/subscriptions/{}", Database::SUBSCRIPTION_MISSING_GUID);
        let expected = ApiError::not_found();

        TestBuilder::new(app, url, expected)
            .method(Method::GET)
            .authorization(true)
            .status(StatusCode::NOT_FOUND)
            .run()
            .await;
    }
}
