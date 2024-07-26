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
            tracing::error!(err = %err, "failed to retrieve user subscriptions");

            Either2::E1(Json(Subscriptions::empty()))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        body::Body,
        http::{header, Method, Request, StatusCode},
        routing::get,
        Router,
    };
    use axum_extra::extract::cookie::Key;
    use headers::{authorization::{Credentials as _, Bearer}, Authorization};
    use tower::ServiceExt as _;

    use crate::{
        config::Config,
        database::{Database, TestData},
        error::Result,
        handlers::test_app,
        Sync,
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
        tracing_subscriber::fmt().init();

        let app = setup_app(TestData::UserData).await;

        let request = Request::builder()
            .method(Method::GET)
            .header(
                header::AUTHORIZATION,
                Authorization::<Bearer>::bearer(Database::TEST_TOKEN).unwrap().0.encode(),
            )
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .uri("/v1/subscriptions")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn unauthorized() {
        let app = setup_app(TestData::UserData).await;

        let request = Request::builder()
            .method(Method::GET)
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .uri("/v1/subscriptions")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
