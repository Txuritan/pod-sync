use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse as _, Response},
};

use crate::{
    extractor::auth::Session,
    handlers::web::{Base, Template},
    SyncState,
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

#[tracing::instrument(skip_all)]
#[autometrics::autometrics]
pub async fn account(
    State(_sync): State<SyncState>,
    session: Session,
    Path(username): Path<String>,
) -> Response {
    if username != session.user.username {
        return (StatusCode::UNAUTHORIZED).into_response();
    }

    (StatusCode::OK, Template(Account::new(Some(session)))).into_response()
}
