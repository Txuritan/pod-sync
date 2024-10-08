use axum::extract::{Path, State};
use axum_extra::either::Either4;
use url::Url;
use uuid::Uuid;

use crate::{
    extractor::auth::Session,
    models::{subscriptions::SubscriptionUpdate, NotFound, Unauthorized, Validation},
    utils::serde::Deserializable,
    SyncState,
};

#[derive(serde::Deserialize)]
pub struct UpdateBody {
    pub new_feed_url: Url,
    pub new_guid: Uuid,
    pub is_subscribed: bool,
}

pub async fn update(
    State(sync): State<SyncState>,
    session: Option<Session>,
    Path(guid): Path<Uuid>,
    Deserializable(encoding, request): Deserializable<UpdateBody>,
) -> Either4<SubscriptionUpdate, Unauthorized, NotFound, Validation> {
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
