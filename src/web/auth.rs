use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse as _, Redirect, Response},
    Form,
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    PrivateCookieJar,
};
use validator::{Validate as _, ValidationErrors};

use crate::{
    extractor::auth::Session,
    web::{Base, Template},
    Sync,
};

pub static SESSION: &str = "pod-sync-session";

#[derive(askama::Template)]
#[template(path = "auth/register.html")]
struct Register {
    base: Base,
}

impl Register {
    fn new(session: Option<Session>, _errors: Option<ValidationErrors>) -> Self {
        Self {
            base: Base::new(session),
        }
    }
}

#[tracing::instrument(skip_all)]
#[autometrics::autometrics]
pub async fn get_register(session: Option<Session>, State(_sync): State<Sync>) -> Response {
    Template(Register::new(session, None)).into_response()
}

#[derive(Debug, validator::Validate, serde::Deserialize)]
pub struct RegisterForm {
    #[validate(length(min = 6, max = 23))]
    username: String,
    #[validate(email)]
    email: String,
    #[validate(length(min = 8, max = 64))]
    password: String,
}

#[tracing::instrument(skip_all)]
#[autometrics::autometrics]
pub async fn post_register(
    session: Option<Session>,
    State(sync): State<Sync>,
    Form(form): Form<RegisterForm>,
) -> Response {
    if let Err(errors) = form.validate() {
        tracing::error!("{}", errors);

        return (
            StatusCode::BAD_REQUEST,
            Template(Register::new(session, Some(errors))),
        )
            .into_response();
    }

    match sync.db.user_get_by_username(&form.username).await {
        Ok(Some(_)) => {
            tracing::error!("user does exist");

            return (
                StatusCode::UNAUTHORIZED,
                Template(Register::new(session, None)),
            )
                .into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Template(Register::new(session, None)),
            )
                .into_response()
        }
    };

    if let Err(err) = sync
        .db
        .user_create(&form.username, &form.email, &form.password)
        .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Template(Register::new(session, None)),
        )
            .into_response();
    };

    Redirect::to("/login").into_response()
}

#[derive(askama::Template)]
#[template(path = "auth/login.html")]
struct Login {
    base: Base,
}

impl Login {
    fn new(session: Option<Session>, _errors: Option<ValidationErrors>) -> Self {
        Self {
            base: Base::new(session),
        }
    }
}

#[tracing::instrument(skip_all)]
#[autometrics::autometrics]
pub async fn get_login(session: Option<Session>, State(_sync): State<Sync>) -> Response {
    Template(Login::new(session, None)).into_response()
}

#[derive(Debug, validator::Validate, serde::Deserialize)]
pub struct LoginForm {
    #[validate(length(min = 6, max = 23))]
    username: String,
    #[validate(length(min = 8, max = 64))]
    password: String,
}

#[tracing::instrument(skip_all)]
#[autometrics::autometrics]
pub async fn post_login(
    jar: PrivateCookieJar,
    session: Option<Session>,
    State(sync): State<Sync>,
    Form(form): Form<LoginForm>,
) -> Response {
    if let Err(errors) = form.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Template(Login::new(session, Some(errors))),
        )
            .into_response();
    }

    let user = match sync.db.user_get_by_username(&form.username).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            tracing::error!("user does not exist");

            return (StatusCode::BAD_REQUEST, Template(Login::new(session, None))).into_response();
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Template(Login::new(session, None)),
            )
                .into_response();
        }
    };

    let (token, expires) = match sync.db.session_crate(&user).await {
        Ok(pair) => pair,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Template(Login::new(session, None)),
            )
                .into_response();
        }
    };

    let mut cookie = Cookie::new(SESSION, token);
    cookie.set_http_only(true);
    cookie.set_path("/");
    cookie.set_same_site(SameSite::Strict);
    cookie.set_expires(expires);

    (jar.add(cookie), Redirect::to("/")).into_response()
}

#[tracing::instrument(skip_all)]
#[autometrics::autometrics]
pub async fn get_logout(jar: PrivateCookieJar) -> Response {
    let Some(session) = jar.get(SESSION) else {
        return Redirect::to("/").into_response();
    };

    (jar.remove(session), Redirect::to("/")).into_response()
}
