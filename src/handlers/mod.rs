mod subscriptions;

use axum::routing::get;

#[rustfmt::skip]
pub fn app() -> axum::Router<crate::Sync> {
    axum::Router::new()
        // Subscriptions
        .route("/v1/subscriptions", get(subscriptions::list).post(subscriptions::add))
        .route("/v1/subscriptions/:guid", get(subscriptions::get).patch(subscriptions::update).delete(subscriptions::delete))
        .route("/v1/deletions/:deletion_id", get(subscriptions::status))
}

#[cfg(test)]
pub async fn test_app<B>(
    args: crate::database::TestData,
    builder: B,
) -> crate::error::Result<axum::Router>
where
    B: FnOnce(axum::Router<crate::Sync>) -> axum::Router<crate::Sync>,
{
    use std::sync::Arc;

    use axum_extra::extract::cookie::Key;

    use crate::{config::Config, database::Database, error::Result, Sync};

    let config = Arc::new(Config::load_test()?);

    let db = Database::new_test(args).await?;

    let state = Sync {
        key: Key::from(&config.cookie_key()?),
        db,
        cfg: config.clone(),
    };

    let router = builder(axum::Router::new()).with_state(state);

    Ok(router)
}
