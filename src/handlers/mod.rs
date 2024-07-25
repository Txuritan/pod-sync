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
