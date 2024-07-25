use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use axum_extra::either::{Either as Either2, Either3, Either4, Either5};
use time::OffsetDateTime;
use url::Url;
use uuid::Uuid;

use crate::{
    extractor::auth::Session,
    models::{
        subscriptions::{
            Deletion, DeletionReceived, FeedArray, NewSubscriptions, Subscription,
            SubscriptionUpdate, Subscriptions,
        },
        Gone, NotFound, Unauthorized, Validation,
    },
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
        return Either2::E2(Unauthorized);
    };
    if !session.validate() {
        return Either2::E2(Unauthorized);
    }

    todo!()
}

pub async fn add(
    State(sync): State<Sync>,
    session: Option<Session>,
    Json(feeds): Json<FeedArray>,
) -> Either3<Json<NewSubscriptions>, Unauthorized, Validation> {
    let Some(session) = session else {
        return Either3::E2(Unauthorized);
    };
    if !session.validate() {
        return Either3::E2(Unauthorized);
    }

    todo!()
}

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

    todo!()
}

#[derive(serde::Deserialize)]
pub struct UpdateBody {
    pub new_feed_url: Url,
    pub new_guid: Uuid,
    pub is_subscribed: bool,
}

pub async fn update(
    State(sync): State<Sync>,
    session: Option<Session>,
    Path(guid): Path<Uuid>,
    Json(request): Json<UpdateBody>,
) -> Either4<Json<SubscriptionUpdate>, Unauthorized, NotFound, Validation> {
    let Some(session) = session else {
        return Either4::E2(Unauthorized);
    };
    if !session.validate() {
        return Either4::E2(Unauthorized);
    }

    todo!()
}

pub async fn delete(
    State(sync): State<Sync>,
    session: Option<Session>,
    Path(guid): Path<Uuid>,
) -> Either4<(StatusCode, Json<DeletionReceived>), Unauthorized, NotFound, Validation> {
    let Some(session) = session else {
        return Either4::E2(Unauthorized);
    };
    if !session.validate() {
        return Either4::E2(Unauthorized);
    }

    todo!()
}

pub async fn status(
    State(sync): State<Sync>,
    session: Option<Session>,
    Path(id): Path<i64>,
) -> Either4<Json<Deletion>, Unauthorized, NotFound, Validation> {
    let Some(session) = session else {
        return Either4::E2(Unauthorized);
    };
    if !session.validate() {
        return Either4::E2(Unauthorized);
    }

    todo!()
}

#[cfg(test)]
mod tests {}
