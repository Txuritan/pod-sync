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

    todo!()
}

#[cfg(test)]
mod tests {}
