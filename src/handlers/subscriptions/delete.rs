use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use axum_extra::either::Either4;
use uuid::Uuid;

use crate::{
    extractor::auth::Session,
    models::{subscriptions::DeletionReceived, NotFound, Unauthorized, Validation},
    Sync,
};

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

#[cfg(test)]
mod tests {}
