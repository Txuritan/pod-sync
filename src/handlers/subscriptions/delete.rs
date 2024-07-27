use axum::extract::{Path, State};
use axum_extra::either::Either4;
use uuid::Uuid;

use crate::{
    extractor::auth::Session,
    models::{subscriptions::DeletionReceived, NotFound, Unauthorized, Validation},
    Sync,
};

#[tracing::instrument(skip_all)]
#[autometrics::autometrics]
pub async fn delete(
    State(sync): State<Sync>,
    session: Option<Session>,
    Path(guid): Path<Uuid>,
) -> Either4<DeletionReceived, Unauthorized, NotFound, Validation> {
    let Some(session) = session else {
        return Either4::E2(Unauthorized);
    };
    if !session.validate() {
        return Either4::E2(Unauthorized);
    }

    let id = sync.db.deletion_create(&session.user, guid).await;

    match id {
        Ok(Some(id)) => Either4::E1(DeletionReceived::new(id)),
        Ok(None) => Either4::E3(NotFound),
        Err(err) => {
            tracing::error!(err = ?err, "Failed to create deletion task");

            Either4::E2(Unauthorized)
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{header, Method, Request, StatusCode},
        routing::delete,
        Router,
    };
    use http_body_util::BodyExt as _;
    use pretty_assertions::assert_eq;
    use tower::ServiceExt as _;

    use crate::{
        database::{test::TestData, Database},
        handlers::test_app,
        models::{subscriptions::DeletionReceived, ApiError},
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

        let request = Request::builder()
            .method(Method::DELETE)
            .header(header::AUTHORIZATION, Database::test_token())
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .uri(format!(
                "/v1/subscriptions/{}",
                Database::SUBSCRIPTION_1_GUID
            ))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = response.into_body().collect().await.unwrap().to_bytes();

        assert!(!body.is_empty());

        let body: DeletionReceived = serde_json::from_slice(&body[..]).unwrap();

        let expected = DeletionReceived::new(4);

        assert_eq!(expected, body);
    }

    #[tokio::test]
    async fn unauthorized() {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();

        let app = setup_app(TestData::UserData).await;

        let request = Request::builder()
            .method(Method::DELETE)
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .uri(format!(
                "/v1/subscriptions/{}",
                Database::SUBSCRIPTION_1_GUID
            ))
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
            .method(Method::DELETE)
            .header(header::AUTHORIZATION, Database::test_token())
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .uri(format!(
                "/v1/subscriptions/{}",
                Database::SUBSCRIPTION_MISSING_GUID
            ))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = response.into_body().collect().await.unwrap().to_bytes();

        assert!(!body.is_empty());

        let body: ApiError = serde_json::from_slice(&body[..]).unwrap();

        assert_eq!(ApiError::not_found(), body);
    }

    // TODO: validation test
}
