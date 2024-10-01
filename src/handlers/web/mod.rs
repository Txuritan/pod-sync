mod auth;
mod user;

use axum::{
    response::{IntoResponse, Response},
    routing,
};
use axum_extra::response::{Css, Html};
use tower_helmet::HelmetLayer;

use crate::extractor::auth::Session;

static STYLE: &str = include_str!("../../../public/style.css");

pub struct Base {
    pub css: &'static str,
    pub session: Option<Session>,
}

impl Base {
    pub fn new(session: Option<Session>) -> Self {
        Self {
            css: STYLE,
            session,
        }
    }
}

pub struct Template<T: askama::Template>(pub T);

impl<T: askama::Template> IntoResponse for Template<T> {
    fn into_response(self) -> axum::response::Response {
        if let Ok(template) = askama::Template::render(&self.0) {
            return Html(template).into_response();
        }

        // TODO: better fallback page
        "failed to render template".into_response()
    }
}

#[rustfmt::skip]
pub fn app() -> axum::Router<crate::SyncState> {
    axum::Router::new()
        .route("/public/style.css", routing::get(get_style))
        .route("/", routing::get(get_index))
        .route("/register", routing::get(auth::get_register).post(auth::post_register))
        .route("/login", routing::get(auth::get_login).post(auth::post_login))
        .route("/logout", routing::get(auth::get_logout))
        .route("/user/:username", routing::get(user::account))
        .layer((
            HelmetLayer::with_defaults(),
        ))
}

async fn get_style() -> Response {
    Css(STYLE).into_response()
}

#[derive(askama::Template)]
#[template(path = "index.html")]
struct Home {
    base: Base,
}

impl Home {
    fn new(session: Option<Session>) -> Self {
        Self {
            base: Base::new(session),
        }
    }
}

async fn get_index(session: Option<Session>) -> Response {
    Template(Home::new(session)).into_response()
}
