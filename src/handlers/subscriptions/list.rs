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
        return Either2::E2(Unauthorized);
    };
    if !session.validate() {
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
mod tests {}
