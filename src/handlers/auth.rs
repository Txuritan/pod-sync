use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::{extract::PrivateCookieJar, typed_header::TypedHeader};
use headers::{authorization::Basic, Authorization};

use crate::{error::Result, extractor::auth::SESSION, Sync};

#[tracing::instrument(skip_all, err)]
#[autometrics::autometrics]
pub async fn login(
    State(sync): State<Sync>,
    jar: PrivateCookieJar,
    auth: Option<TypedHeader<Authorization<Basic>>>,
    Path(username): Path<String>,
) -> Result<Response> {
    if let Some(TypedHeader(auth)) = auth {
        let Some(user) = sync.db.user_get_by_username(auth.username()).await? else {
            return Ok(StatusCode::BAD_REQUEST.into_response());
        };

        if !user.verify(auth.password()) {
            return Ok(StatusCode::BAD_REQUEST.into_response());
        }

        let cookie = sync.db.user_login(&user).await?;

        return Ok((StatusCode::OK, jar.add(cookie)).into_response());
    }

    let Some(session) = jar.get(SESSION) else {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    };

    if sync.db.user_get_by_username(&username).await?.is_none() {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    };

    return if sync.db.user_get_session(session.value()).await?.is_some() {
        Ok(StatusCode::OK.into_response())
    } else {
        Ok(StatusCode::UNAUTHORIZED.into_response())
    };
}

#[tracing::instrument(skip_all, err)]
#[autometrics::autometrics]
pub async fn logout(jar: PrivateCookieJar) -> Result<Response> {
    let Some(session) = jar.get(SESSION) else {
        return Ok(StatusCode::OK.into_response());
    };

    Ok((StatusCode::OK, jar.remove(session)).into_response())
}

#[cfg(test)]
mod tests {}
