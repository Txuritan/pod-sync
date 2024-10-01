pub mod add;
pub mod delete;
pub mod get;
pub mod list;
pub mod status;
pub mod update;

use axum::routing;

#[rustfmt::skip]
pub fn app() -> axum::Router<crate::SyncState> {
    axum::Router::new()
        // Subscriptions
        .route("/v1/subscriptions", routing::get(list::list).post(add::add))
        .route("/v1/subscriptions/:guid", routing::get(get::get).patch(update::update).delete(delete::delete))
        .route("/v1/deletions/:deletion_id", routing::get(status::status))
}
