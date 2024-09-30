mod subscriptions;
mod web;

use crate::SyncState;

#[rustfmt::skip]
pub fn app(state: SyncState) -> axum::Router {
    axum::Router::new()
        .merge(subscriptions::app())
        .merge(web::app())
        .with_state(state.clone())
}

#[cfg(test)]
pub async fn test_app<B>(
    pool: sqlx::SqlitePool,
    builder: B,
) -> anyhow::Result<axum::Router>
where
    B: FnOnce(axum::Router<SyncState>) -> axum::Router<SyncState>,
{
    use tower_http::trace::TraceLayer;

    let state = SyncState::new_test(pool).await?;

    let router = builder(axum::Router::new())
        .with_state(state)
        .layer((TraceLayer::new_for_http(),));

    Ok(router)
}
