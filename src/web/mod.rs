mod auth;
mod user;

use axum::{
    response::{IntoResponse, Response},
    routing::get,
};
use axum_extra::response::{Css, Html};

use crate::{
    error::{Error, Result},
    extractor::auth::{MaybeAuthenticated, Session},
};

static STYLE: &str = include_str!("../../public/style.css");

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
        askama::Template::render(&self.0)
            .map_err(Error::from)
            .map(Html)
            .into_response()
    }
}

#[rustfmt::skip]
pub fn app() -> axum::Router<crate::Sync> {
    axum::Router::new()
        .route("/public/style.css", get(get_style))
        .route("/", get(get_index))
        .route("/register", get(auth::get_register).post(auth::post_register))
        .route("/login", get(auth::get_login).post(auth::post_login))
        .route("/logout", get(auth::get_logout))
        .route("/user/:id", get(user::account))
        .route("/user/:id/devices", get(user::devices))
}

async fn get_style() -> Result<Response> {
    Ok(Css(STYLE).into_response())
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

async fn get_index(MaybeAuthenticated(session): MaybeAuthenticated) -> Result<Response> {
    Ok(Template(Home::new(session)).into_response())
}
