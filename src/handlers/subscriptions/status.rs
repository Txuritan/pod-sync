use axum::extract::{Path, State};
use axum_extra::either::Either4;

use crate::{
    database::tasks::DeletionId,
    extractor::auth::Session,
    models::{subscriptions::Deletion, NotFound, Unauthorized, Validation},
    Sync,
};

#[tracing::instrument(skip_all)]
#[autometrics::autometrics]
pub async fn status(
    State(sync): State<Sync>,
    session: Option<Session>,
    Path(id): Path<DeletionId>,
) -> Either4<Deletion, Unauthorized, NotFound, Validation> {
    let Some(session) = session else {
        return Either4::E2(Unauthorized);
    };
    if !session.validate() {
        return Either4::E2(Unauthorized);
    }

    let status = sync.db.deletion_get(&session.user, id).await;

    match status {
        Ok(Some(status)) => Either4::E1(status),
        Ok(None) => Either4::E3(NotFound),
        Err(err) => {
            tracing::error!(err = ?err, "Failed to get deletion task status");

            Either4::E2(Unauthorized)
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{header, Method, Request, StatusCode},
        routing::get,
        Router,
    };
    use http_body_util::BodyExt as _;
    use pretty_assertions::assert_eq;
    use tower::ServiceExt as _;

    use crate::{
        database::{test::TestData, Database},
        handlers::test_app,
        models::{subscriptions::Deletion, ApiError},
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

        let request = Request::builder()
            .method(Method::GET)
            .header(header::AUTHORIZATION, Database::test_token())
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .uri(format!("/v1/deletions/{}", Database::DELETION_PENDING_ID))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();

        assert!(!body.is_empty());

        let body: Deletion = serde_json::from_slice(&body[..]).unwrap();

        let expected = Deletion::pending(Database::DELETION_PENDING_ID);

        assert_eq!(expected, body);
    }

    #[tokio::test]
    async fn ok_success() {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();

        let app = setup_app(TestData::UserData).await;

        let request = Request::builder()
            .method(Method::GET)
            .header(header::AUTHORIZATION, Database::test_token())
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .uri(format!("/v1/deletions/{}", Database::DELETION_SUCCESS_ID))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();

        assert!(!body.is_empty());

        let body: Deletion = serde_json::from_slice(&body[..]).unwrap();

        let expected = Deletion::success(Database::DELETION_SUCCESS_ID);

        assert_eq!(expected, body);
    }

    #[tokio::test]
    async fn ok_failure() {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();

        let app = setup_app(TestData::UserData).await;

        let request = Request::builder()
            .method(Method::GET)
            .header(header::AUTHORIZATION, Database::test_token())
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .uri(format!("/v1/deletions/{}", Database::DELETION_FAILURE_ID))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();

        assert!(!body.is_empty());

        let body: Deletion = serde_json::from_slice(&body[..]).unwrap();

        let expected = Deletion::failure(Database::DELETION_FAILURE_ID);

        assert_eq!(expected, body);
    }

    #[tokio::test]
    async fn unauthorized() {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();

        let app = setup_app(TestData::UserData).await;

        let request = Request::builder()
            .method(Method::GET)
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .uri(format!("/v1/deletions/{}", Database::DELETION_PENDING_ID))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let body = response.into_body().collect().await.unwrap().to_bytes();

        assert!(!body.is_empty());

        let body: ApiError = serde_json::from_slice(&body[..]).unwrap();

        assert_eq!(ApiError::unauthorized(), body);
    }

    #[tokio::test]
    async fn not_found() {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();

        let app = setup_app(TestData::UserData).await;

        let request = Request::builder()
            .method(Method::GET)
            .header(header::AUTHORIZATION, Database::test_token())
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .uri(format!("/v1/deletions/{}", Database::DELETION_MISSING_ID))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = response.into_body().collect().await.unwrap().to_bytes();

        assert!(!body.is_empty());

        let body: ApiError = serde_json::from_slice(&body[..]).unwrap();

        assert_eq!(ApiError::not_found(), body);
    }
}
