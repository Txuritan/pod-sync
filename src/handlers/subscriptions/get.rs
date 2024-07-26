use axum::{
    extract::{Path, State},
    Json,
};
use axum_extra::either::Either5;
use uuid::Uuid;

use crate::{
    extractor::auth::Session,
    models::{subscriptions::Subscription, Gone, NotFound, Unauthorized, Validation},
    Sync,
};

#[tracing::instrument(skip_all)]
#[autometrics::autometrics]
pub async fn get(
    State(sync): State<Sync>,
    session: Option<Session>,
    Path(guid): Path<Uuid>,
) -> Either5<Json<Subscription>, Unauthorized, NotFound, Validation, Gone> {
    let Some(session) = session else {
        return Either5::E2(Unauthorized);
    };
    if !session.validate() {
        return Either5::E2(Unauthorized);
    }

    let subscription = match sync.db.subscription_get_by_guid(&session.user, guid).await {
        Ok(Some(subscription)) => subscription,
        Ok(None) => return Either5::E3(NotFound),
        Err(err) => {
            tracing::error!(err = ?err, "Failed to retrieve user subscription");

            return Either5::E2(Unauthorized);
        }
    };

    if subscription.deleted.is_some() {
        return Either5::E5(Gone);
    }

    Either5::E1(Json(subscription))
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
    use tower::ServiceExt as _;
    use tracing::Level;
    use url::Url;

    use crate::{
        database::{test::TestData, Database},
        handlers::test_app,
        models::{subscriptions::Subscription, ApiError},
    };

    async fn setup_app(args: TestData) -> Router {
        test_app(args, |router| {
            router.route("/v1/subscriptions/:guid", get(super::get))
        })
        .await
        .expect("failed to setup app")
    }

    #[tokio::test]
    async fn ok() {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();

        let app = setup_app(TestData::UserData).await;

        let request = Request::builder()
            .method(Method::GET)
            .header(header::AUTHORIZATION, Database::test_token())
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .uri(format!(
                "/v1/subscriptions/{}",
                Database::SUBSCRIPTION_1_GUID
            ))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();

        assert!(!body.is_empty());

        let body: Subscription = serde_json::from_slice(&body[..]).unwrap();

        let expected = Subscription {
            feed_url: Url::parse(Database::SUBSCRIPTION_1_FEED).unwrap(),
            guid: Database::SUBSCRIPTION_1_GUID,
            is_subscribed: true,
            subscription_changed: None,
            new_guid: None,
            guid_changed: None,
            deleted: None,
        };

        assert_eq!(expected, body);
    }

    #[tokio::test]
    async fn unauthorized() {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();

        let app = setup_app(TestData::UserData).await;

        let request = Request::builder()
            .method(Method::GET)
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

    // TODO: validation test

    #[tokio::test]
    async fn not_found() {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();

        let app = setup_app(TestData::UserData).await;

        let request = Request::builder()
            .method(Method::GET)
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
}
