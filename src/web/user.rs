use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse as _, Redirect, Response},
    Form,
};

use crate::{
    database::{Device, DeviceType, DeviceUpdate},
    error::Result,
    extractor::auth::{RequireAuthentication, Session},
    web::{Base, Template},
    Sync,
};

#[derive(askama::Template)]
#[template(path = "user/index.html")]
struct Account {
    base: Base,
    devices: Vec<Device>,
}

impl Account {
    fn new(session: Option<Session>, devices: Vec<Device>) -> Self {
        Self {
            base: Base::new(session),
            devices,
        }
    }
}

#[tracing::instrument(skip_all, err)]
#[autometrics::autometrics]
pub async fn account(
    State(sync): State<Sync>,
    RequireAuthentication(session): RequireAuthentication,
    Path(username): Path<String>,
) -> Result<Response> {
    if username != session.user.username {
        return Ok((StatusCode::UNAUTHORIZED).into_response());
    }

    let devices = sync.db.get_devices(&session.user).await?;

    Ok((
        StatusCode::OK,
        Template(Account::new(Some(session), devices)),
    )
        .into_response())
}

#[derive(Clone, serde::Deserialize)]
pub struct DeviceAdd {
    pub name: String,
    pub caption: Option<String>,
    #[serde(rename = "type")]
    pub typ: Option<DeviceType>,
}

#[tracing::instrument(skip_all, err)]
#[autometrics::autometrics]
pub async fn device_add(
    State(sync): State<Sync>,
    RequireAuthentication(session): RequireAuthentication,
    Path(username): Path<String>,
    Form(add): Form<DeviceAdd>,
) -> Result<Response> {
    if username != session.user.username {
        return Ok((StatusCode::UNAUTHORIZED).into_response());
    }

    let DeviceAdd { name, caption, typ } = add;
    let update = DeviceUpdate { caption, typ };

    sync.db.update_device(&session.user, &name, update).await?;

    Ok(Redirect::to(&format!("/user/{}", username)).into_response())
}

#[tracing::instrument(skip_all, err)]
#[autometrics::autometrics]
pub async fn device_remove(
    State(sync): State<Sync>,
    RequireAuthentication(session): RequireAuthentication,
    Path((username, device_name)): Path<(String, String)>,
) -> Result<Response> {
    if username != session.user.username {
        return Ok((StatusCode::UNAUTHORIZED).into_response());
    }

    let Some(device_id) = sync.db.get_device_id(&session.user, &device_name).await? else {
        return Ok(StatusCode::BAD_REQUEST.into_response());
    };

    sync.db.remove_device(&session.user, device_id).await?;

    Ok(Redirect::to(&format!("/user/{}", username)).into_response())
}
