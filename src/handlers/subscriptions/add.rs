use axum::{extract::State, Json};
use axum_extra::either::Either3;

use crate::{
    extractor::auth::Session,
    models::{
        subscriptions::{FeedArray, NewSubscriptions},
        Unauthorized, Validation,
    },
    Sync,
};

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

#[cfg(test)]
mod tests {}
