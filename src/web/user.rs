use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse as _, Response},
};

use crate::{
    error::Result,
    extractor::auth::Session,
    web::{Base, Template},
    Sync,
};

#[derive(askama::Template)]
#[template(path = "user/index.html")]
struct Account {
    base: Base,
}

impl Account {
    fn new(session: Option<Session>) -> Self {
        Self {
            base: Base::new(session),
        }
    }
}

#[tracing::instrument(skip_all, err)]
#[autometrics::autometrics]
pub async fn account(
    State(_sync): State<Sync>,
    session: Session,
    Path(username): Path<String>,
) -> Result<Response> {
    if username != session.user.username {
        return Ok((StatusCode::UNAUTHORIZED).into_response());
    }

    Ok((StatusCode::OK, Template(Account::new(Some(session)))).into_response())
}
