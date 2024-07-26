use axum::{
    extract::{Query, State},
    Json,
};
use axum_extra::either::Either as Either2;
use time::OffsetDateTime;

use crate::{
    extractor::auth::Session,
    models::{subscriptions::Subscriptions, Unauthorized},
    Sync,
};

#[derive(serde::Deserialize)]
pub struct ListParams {
    pub since: Option<OffsetDateTime>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

pub async fn list(
    State(sync): State<Sync>,
    session: Option<Session>,
    Query(params): Query<ListParams>,
) -> Either2<Json<Subscriptions>, Unauthorized> {
    let Some(session) = session else {
        tracing::info!("no session");
        return Either2::E2(Unauthorized);
    };
    if !session.validate() {
        tracing::info!("session invalid");
        return Either2::E2(Unauthorized);
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
        Ok(Some(subscriptions)) => Either2::E1(Json(subscriptions)),
        Ok(None) => Either2::E1(Json(Subscriptions::empty())),
        Err(err) => {
            tracing::error!(err = ?err, "Failed to retrieve user subscriptions");

            Either2::E1(Json(Subscriptions::empty()))
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
    use tower::ServiceExt as _;
    use url::Url;

    use crate::{
        database::{Database, TestData},
        handlers::test_app,
        models::{subscriptions::{Subscription, Subscriptions}, ApiError},
    };

    async fn setup_app(args: TestData) -> Router {
        test_app(args, |router| {
            router.route("/v1/subscriptions", get(super::list))
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
            .uri("/v1/subscriptions")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();

        assert!(!body.is_empty());

        let body: Subscriptions = serde_json::from_slice(&body[..]).unwrap();

        let expected = Subscriptions {
            total: 1,
            page: 1,
            per_page: 50,
            next: None,
            previous: None,
            subscriptions: vec![
                Subscription {
                    feed_url: Url::parse(Database::TEST_SUBSCRIPTION_FEED).unwrap(),
                    guid: Database::TEST_SUBSCRIPTION_GUID,
                    is_subscribed: true,
                    subscription_changed: None,
                    new_guid: None,
                    guid_changed: None,
                    deleted: None,
                },
            ],
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
            .uri("/v1/subscriptions")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let body = response.into_body().collect().await.unwrap().to_bytes();

        assert!(!body.is_empty());

        let body: ApiError = serde_json::from_slice(&body[..]).unwrap();

        assert_eq!(ApiError::unauthorized(), body);
    }
}
