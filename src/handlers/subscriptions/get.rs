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

#[cfg(test)]
mod tests {}
