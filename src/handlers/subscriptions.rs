use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse as _, Response},
    Json,
};
use time::OffsetDateTime;

use crate::{database::Database, error::Result, extractor::auth::RequireAuthentication, Sync};

#[derive(Debug, serde::Deserialize)]
pub struct ChangesRequest {
    pub add: Vec<String>,
    pub remove: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct ChangesResponse {
    pub add: Vec<String>,
    pub remove: Vec<String>,
    pub timestamp: OffsetDateTime,
}

impl Database {}

pub async fn get_all(
    RequireAuthentication(session): RequireAuthentication,
    State(state): State<Sync>,
    Path(username): Path<String>,
) -> Result<Response> {
    if session.user.username != username {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    todo!()
}

pub async fn get_of_device(
    RequireAuthentication(session): RequireAuthentication,
    State(state): State<Sync>,
    Path((username, device_id)): Path<(String, String)>,
) -> Result<Response> {
    if session.user.username != username {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    todo!()
}

pub async fn upload_of_device(
    RequireAuthentication(session): RequireAuthentication,
    State(state): State<Sync>,
    Path((username, device_id)): Path<(String, String)>,
    Json(changes): Json<ChangesRequest>,
) -> Result<Response> {
    if session.user.username != username {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    todo!()
}

/// # Get Subscription Changes
///
/// `GET /api/2/subscriptions/(username)/(device_id).json`
///
///   - Requires HTTP authentication
///   - Since 2.0
///
/// This API call retrieves the subscription changes since the timestamp
/// provided in the since parameter. Its value SHOULD be timestamp value
/// from the previous call to this API endpoint. If there has been no
/// previous call, the client SHOULD use 0.
///
/// The response format is the same as the upload format: A dictionary with
/// two keys “add” and “remove” where the value for each key is a list of
/// URLs that should be added or removed. The timestamp SHOULD be stored by
/// the client in order to provide it in the since parameter in the next
/// request.
///
/// ## Parameters
///
///   - **username** - username for which subscriptions should be returned
///   - **device_id** - see Devices
///
/// ## Query Parameters
///
///   - **since** - the `timestamp` value of the last response
///
/// ## Example response:
///
/// In case nothing has changed, the server returns something like the
/// following JSON content.
///
/// ```json
/// {
///    "add": [],
///    "remove": [],
///    "timestamp": 12347
/// }
/// ```
pub async fn get_changes(
    RequireAuthentication(session): RequireAuthentication,
    State(state): State<Sync>,
    Path((username, device_id)): Path<(String, String)>,
) -> Result<Response> {
    if session.user.username != username {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    todo!()
}

/// # Upload Subscription Changes
///
/// `POST /api/2/subscriptions/(username)/(device_id).json`
///
///   - Requires HTTP authentication
///   - Since 2.0
///
/// Only deltas are supported here. Timestamps are not supported, and are
/// issued by the server.
///
/// In positive responses the server returns a timestamp/ID that can be
/// used for requesting changes since this upload in a subsequent API call.
/// In addition, the server sends a list of URLs that have been rewritten
/// (sanitized, see bug:747) as a list of tuples with the key “update_urls”.
/// The client SHOULD parse this list and update the local subscription list
/// accordingly (the server only sanitizes the URL, so the semantic “content”
/// should stay the same and therefore the client can simply update the URL
/// value locally and use it for future updates.
///
/// URLs that are not allowed (currently all URLs that don’t start with either
/// http or https) are rewritten to the empty string and are ignored by the
/// Web-service.
///
/// ## Parameters
///
///   - **username** - username for which subscriptions should be returned
///   - **device_id** - see Devices
///
/// ## Status Codes
///
///   - **200 Ok** - the subscriptions have been updated
///   - **400 Bad Request** - the same feed has been added and removed in the same request
///
/// ## Example request
///
/// ```json
/// {
///     "add": ["http://example.com/feed.rss", "http://example.org/podcast.php"],
///     "remove": ["http://example.net/foo.xml"]
/// }
/// ```
///
/// ## Example response
///
/// ```json
/// {
///     "timestamp": 1337,
///     "update_urls": [
///         [
///             "http://feeds2.feedburner.com/LinuxOutlaws?format=xml",
///             "http://feeds.feedburner.com/LinuxOutlaws"
///         ],
///         [
///             "http://example.org/podcast.rss ",
///             "http://example.org/podcast.rss"
///         ]
///     ]
/// }
/// ```
pub async fn upload_changes(
    RequireAuthentication(session): RequireAuthentication,
    State(state): State<Sync>,
    Path((username, device_id)): Path<(String, String)>,
    Json(changes): Json<ChangesRequest>,
) -> Result<Response> {
    if session.user.username != username {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    todo!()
}
