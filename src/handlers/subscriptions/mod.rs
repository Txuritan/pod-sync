pub mod add;
pub mod delete;
pub mod get;
pub mod list;
pub mod status;
pub mod update;

#[rustfmt::skip]
pub fn app() -> axum::Router<crate::SyncState> {
    use axum::routing::get as _get;

    axum::Router::new()
        // Subscriptions
        .route("/v1/subscriptions", _get(list::list).post(add::add))
        .route("/v1/subscriptions/:guid", _get(get::get).patch(update::update).delete(delete::delete))
        .route("/v1/deletions/:deletion_id", _get(status::status))
}
