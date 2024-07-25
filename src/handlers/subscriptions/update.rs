use axum::{
    extract::{Path, State},
    Json,
};
use axum_extra::either::Either4;
use url::Url;
use uuid::Uuid;

use crate::{
    extractor::auth::Session,
    models::{subscriptions::SubscriptionUpdate, NotFound, Unauthorized, Validation},
    Sync,
};

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

#[cfg(test)]
mod tests {}
