mod subscriptions;
mod web;

#[rustfmt::skip]
pub fn app() -> axum::Router<crate::Sync> {
    axum::Router::new()
        .merge(subscriptions::app())
        .merge(web::app())
}

#[cfg(test)]
pub async fn test_app<B>(
    args: crate::database::test::TestData,
    builder: B,
) -> anyhow::Result<axum::Router>
where
    B: FnOnce(axum::Router<crate::Sync>) -> axum::Router<crate::Sync>,
{
    use std::sync::Arc;

    use axum_extra::extract::cookie::Key;
    use tower_http::trace::TraceLayer;

    use crate::{config::Config, database::Database, Sync};

    let config = Arc::new(Config::load_test()?);

    let db = Database::new_test(args).await?;

    let state = Sync {
        key: Key::from(&config.cookie_key()?),
        db,
        cfg: config.clone(),
    };

    let router = builder(axum::Router::new())
        .with_state(state)
        .layer((TraceLayer::new_for_http(),));

    Ok(router)
}
