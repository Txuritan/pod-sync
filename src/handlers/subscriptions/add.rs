use axum::{extract::State, Json};
use axum_extra::either::Either3;

use crate::{
    extractor::auth::Session,
    models::{
        subscriptions::{AddSubscriptions, NewSubscriptions},
        Unauthorized, Validation,
    },
    Sync,
};

pub async fn add(
    State(sync): State<Sync>,
    session: Option<Session>,
    Json(add): Json<AddSubscriptions>,
) -> Either3<Json<NewSubscriptions>, Unauthorized, Validation> {
    // let Some(session) = session else {
    //     return Either3::E2(Unauthorized);
    // };
    // if !session.validate() {
    //     return Either3::E2(Unauthorized);
    // }

    for feed in add.subscriptions {
        tracing::info!(url = %feed.feed_url, "Feed");
    }

    Either3::E1(Json(NewSubscriptions {
        success: vec![],
        failure: vec![],
    }))
}

#[cfg(test)]
mod tests {}
