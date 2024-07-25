mod auth;
mod device;
mod directory;
mod episode;
mod favorites;
mod podcast;
mod settings;
mod subscriptions;
mod suggestions;
mod sync;

use axum::routing::{get, post};

#[rustfmt::skip]
pub fn app() -> axum::Router<crate::Sync> {
    axum::Router::new()
        // authentication
        .route("/api/2/auth/:username/login/.json", post(auth::login))
        .route("/api/2/auth/:username/logout/.json", post(auth::logout))
        // directory
        // suggestions
        // device
        .route("/api/2/devices/:username/:device_name/.json", post(device::update))
        .route("/api/2/devices/:username/.json", get(device::list))
        .route("/api/2/updates/:username/:device_name/.json", get(device::updates))
        // subscriptions
        .route("/subscriptions/:username/:device_name/.json", get(subscriptions::get_of_device).put(subscriptions::upload_of_device))
        .route("/subscriptions/:username/.json", get(subscriptions::get_all))
        .route("/api/2/subscriptions/:username/:device_name/.json", get(subscriptions::get_changes).post(subscriptions::upload_changes))
    // episode
        // r.Get("/api/2/episodes/{username}.{format}", episodeAPI.HandleEpisodeAction)
        // r.Post("/api/2/episodes/{username}.{format}", episodeAPI.HandleUploadEpisodeAction)
    // lists
    // settings
    // favorites
    // synchronization
        // r.Get("/api/2/sync-devices/{username}.json", syncAPI.HandleGetSync)
        // r.Post("/api/2/sync-devices/{username}.json", syncAPI.HandlePostSync)
    // parametrization
}
