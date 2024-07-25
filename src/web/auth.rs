use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse as _, Redirect, Response},
    Form,
};
use axum_extra::extract::PrivateCookieJar;
use validator::{Validate as _, ValidationErrors};

use crate::{
    error::Result,
    extractor::auth::{MaybeAuthenticated, Session, SESSION},
    web::{Base, Template},
    Sync,
};

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

#[tracing::instrument(skip_all, err)]
#[autometrics::autometrics]
pub async fn get_register(
    MaybeAuthenticated(session): MaybeAuthenticated,
    State(_sync): State<Sync>,
) -> Result<Response> {
    Ok(Template(Register::new(session, None)).into_response())
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

#[tracing::instrument(skip_all, err)]
#[autometrics::autometrics]
pub async fn post_register(
    MaybeAuthenticated(session): MaybeAuthenticated,
    State(sync): State<Sync>,
    Form(form): Form<RegisterForm>,
) -> Result<Response> {
    if let Err(errors) = form.validate() {
        tracing::error!("{}", errors);
        return Ok((
            StatusCode::BAD_REQUEST,
            Template(Register::new(session, Some(errors))),
        )
            .into_response());
    }

    if sync
        .db
        .user_get_by_username(&form.username)
        .await?
        .is_some()
    {
        tracing::error!("user does exist");
        return Ok((
            StatusCode::UNAUTHORIZED,
            Template(Register::new(session, None)),
        )
            .into_response());
    }

    sync.db
        .user_create(&form.username, &form.email, &form.password)
        .await?;

    Ok(Redirect::to("/login").into_response())
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

#[tracing::instrument(skip_all, err)]
#[autometrics::autometrics]
pub async fn get_login(
    MaybeAuthenticated(session): MaybeAuthenticated,
    State(_sync): State<Sync>,
) -> Result<Response> {
    Ok(Template(Login::new(session, None)).into_response())
}

#[derive(Debug, validator::Validate, serde::Deserialize)]
pub struct LoginForm {
    #[validate(length(min = 6, max = 23))]
    username: String,
    #[validate(length(min = 8, max = 64))]
    password: String,
}

#[tracing::instrument(skip_all, err)]
#[autometrics::autometrics]
pub async fn post_login(
    jar: PrivateCookieJar,
    MaybeAuthenticated(session): MaybeAuthenticated,
    State(sync): State<Sync>,
    Form(form): Form<LoginForm>,
) -> Result<Response> {
    if let Err(errors) = form.validate() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Template(Login::new(session, Some(errors))),
        )
            .into_response());
    }

    let Some(user) = sync.db.user_get_by_username(&form.username).await? else {
        tracing::error!("user does not exist");
        return Ok((StatusCode::BAD_REQUEST, Template(Login::new(session, None))).into_response());
    };

    let cookie = sync.db.user_login(&user).await?;

    Ok((jar.add(cookie), Redirect::to("/")).into_response())
}

#[tracing::instrument(skip_all, err)]
#[autometrics::autometrics]
pub async fn get_logout(jar: PrivateCookieJar) -> Result<Response> {
    let Some(session) = jar.get(SESSION) else {
        return Ok(Redirect::to("/").into_response());
    };

    Ok((jar.remove(session), Redirect::to("/")).into_response())
}
