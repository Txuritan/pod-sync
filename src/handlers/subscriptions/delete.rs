use axum::extract::{Path, State};
use axum_extra::either::Either5;
use uuid::Uuid;

use crate::{
    extractor::auth::Session,
    models::{subscriptions::DeletionReceived, InternalError, NotFound, Unauthorized, Validation},
    Sync,
};

#[tracing::instrument(skip_all)]
#[autometrics::autometrics]
pub async fn delete(
    State(sync): State<Sync>,
    session: Option<Session>,
    Path(guid): Path<Uuid>,
) -> Either5<DeletionReceived, Unauthorized, NotFound, Validation, InternalError> {
    let Some(session) = session else {
        return Either5::E2(Unauthorized);
    };
    if !session.validate() {
        return Either5::E2(Unauthorized);
    }

    let id = sync.db.deletion_create(&session.user, guid).await;

    match id {
        Ok(Some(id)) => Either5::E1(DeletionReceived::new(id)),
        Ok(None) => Either5::E3(NotFound),
        Err(err) => {
            tracing::error!(err = ?err, "Failed to create deletion task");

            Either5::E5(InternalError)
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::{
        http::{Method, StatusCode},
        routing::delete,
        Router,
    };

    use crate::{
        database::{test::TestData, Database},
        handlers::test_app,
        models::{subscriptions::DeletionReceived, ApiError},
        utils::test::TestBuilder,
    };

    async fn setup_app(args: TestData) -> Router {
        test_app(args, |router| {
            router.route("/v1/subscriptions/:guid", delete(super::delete))
        })
        .await
        .expect("failed to setup app")
    }

    #[tokio::test]
    async fn created() {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();

        let app = setup_app(TestData::UserData).await;
        let url = format!("/v1/subscriptions/{}", Database::SUBSCRIPTION_1_GUID);
        let expected = DeletionReceived::new(4);

        TestBuilder::new(app, url, expected)
            .method(Method::DELETE)
            .authorization(true)
            .status(StatusCode::CREATED)
            .run()
            .await;
    }

    #[tokio::test]
    async fn unauthorized() {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();

        let app = setup_app(TestData::UserData).await;
        let url = format!("/v1/subscriptions/{}", Database::SUBSCRIPTION_1_GUID);
        let expected = ApiError::unauthorized();

        TestBuilder::new(app, url, expected)
            .method(Method::DELETE)
            .status(StatusCode::UNAUTHORIZED)
            .run()
            .await;
    }

    #[tokio::test]
    async fn not_found() {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();

        let app = setup_app(TestData::UserData).await;
        let url = format!("/v1/subscriptions/{}", Database::SUBSCRIPTION_MISSING_GUID);
        let expected = ApiError::not_found();

        TestBuilder::new(app, url, expected)
            .method(Method::DELETE)
            .authorization(true)
            .status(StatusCode::NOT_FOUND)
            .run()
            .await;
    }

    // TODO: validation test
}
