use axum::extract::{Path, State};
use axum_extra::either::Either5;

use crate::{
    database::tasks::DeletionId,
    extractor::auth::Session,
    models::{subscriptions::Deletion, InternalError, NotFound, Unauthorized, Validation},
    Sync,
};

#[tracing::instrument(skip_all)]
#[autometrics::autometrics]
pub async fn status(
    State(sync): State<Sync>,
    session: Option<Session>,
    Path(id): Path<DeletionId>,
) -> Either5<Deletion, Unauthorized, NotFound, Validation, InternalError> {
    let Some(session) = session else {
        return Either5::E2(Unauthorized);
    };
    if !session.validate() {
        return Either5::E2(Unauthorized);
    }

    let status = sync.db.deletion_get(&session.user, id).await;

    match status {
        Ok(Some(status)) => Either5::E1(status),
        Ok(None) => Either5::E3(NotFound),
        Err(err) => {
            tracing::error!(err = ?err, "Failed to get deletion task status");

            Either5::E5(InternalError)
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

    use crate::{
        database::{test::TestData, Database},
        handlers::test_app,
        models::{subscriptions::Deletion, ApiError},
        utils::test::TestBuilder,
    };

    async fn setup_app(args: TestData) -> Router {
        test_app(args, |router| {
            router.route("/v1/deletions/:deletion_id", get(super::status))
        })
        .await
        .expect("failed to setup app")
    }

    #[tokio::test]
    async fn ok_pending() {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();

        let app = setup_app(TestData::UserData).await;
        let url = format!("/v1/deletions/{}", Database::DELETION_PENDING_ID);
        let expected = Deletion::pending(Database::DELETION_PENDING_ID);

        TestBuilder::new(app, url, expected)
            .method(Method::GET)
            .authorization(true)
            .status(StatusCode::OK)
            .run()
            .await;
    }

    #[tokio::test]
    async fn ok_success() {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();

        let app = setup_app(TestData::UserData).await;
        let url = format!("/v1/deletions/{}", Database::DELETION_SUCCESS_ID);
        let expected = Deletion::success(Database::DELETION_SUCCESS_ID);

        TestBuilder::new(app, url, expected)
            .method(Method::GET)
            .authorization(true)
            .status(StatusCode::OK)
            .run()
            .await;
    }

    #[tokio::test]
    async fn ok_failure() {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();

        let app = setup_app(TestData::UserData).await;
        let url = format!("/v1/deletions/{}", Database::DELETION_FAILURE_ID);
        let expected = Deletion::failure(Database::DELETION_FAILURE_ID);

        TestBuilder::new(app, url, expected)
            .method(Method::GET)
            .authorization(true)
            .status(StatusCode::OK)
            .run()
            .await;
    }

    #[tokio::test]
    async fn unauthorized() {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();

        let app = setup_app(TestData::UserData).await;
        let url = format!("/v1/deletions/{}", Database::DELETION_PENDING_ID);
        let expected = ApiError::unauthorized();

        TestBuilder::new(app, url, expected)
            .method(Method::GET)
            .status(StatusCode::UNAUTHORIZED)
            .run()
            .await;
    }

    #[tokio::test]
    async fn not_found() {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();

        let app = setup_app(TestData::UserData).await;
        let url = format!("/v1/deletions/{}", Database::DELETION_MISSING_ID);
        let expected = ApiError::not_found();

        TestBuilder::new(app, url, expected)
            .method(Method::GET)
            .authorization(true)
            .status(StatusCode::NOT_FOUND)
            .run()
            .await;
    }
}
