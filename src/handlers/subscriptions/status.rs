use axum::{
    extract::{Path, State},
    Json,
};
use axum_extra::either::Either4;

use crate::{
    extractor::auth::Session,
    models::{subscriptions::Deletion, NotFound, Unauthorized, Validation},
    Sync,
};

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
