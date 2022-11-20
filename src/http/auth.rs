use axum::{http, response, routing::get, Extension, Json, Router};
use sqlx::PgPool;

use serde::Deserialize;
use thiserror::Error;

use crate::{
    http::session::{self, Session},
    password,
};

pub(crate) fn router() -> Router
{
    Router::new().route("/auth", get(fetch_auth_session).post(create_auth_session))
}

#[derive(Deserialize)]
pub(crate) struct CreateAuthSession
{
    username: String,
    password: String,
}

async fn fetch_auth_session(user_id: session::extractor::UserId) -> Result<String>
{
    match user_id {
        session::extractor::UserId::Found(user_id) => Ok(user_id.to_string()),
        session::extractor::UserId::NotFound => Err(Error::MustBeAuthenticated),
    }
}

async fn create_auth_session(
    db_pool: Extension<PgPool>,
    session_store: Extension<session::Store>,
    Json(req): Json<CreateAuthSession>,
) -> Result<(http::HeaderMap, http::StatusCode)>
{
    let CreateAuthSession { username, password } = req;

    let user = sqlx::query!(
        r#"select user_id, password from users where username = $1"#,
        username
    )
    .fetch_optional(&*db_pool)
    .await?;

    match user {
        Some(user) => {
            let password_is_correct = password::verify(password, user.password).await?;

            if password_is_correct {
                let mut session = Session::new();
                session.insert("user_id", user.user_id).await?;
                // SAFETY: This cannot fail as store_session propagates `None`
                // upon a `None` field for the session's cookie value, which
                // will never be empty as we create the session above and never
                // mutate its cookie value
                let cookie = session_store.store_session(session).await?.unwrap();

                let mut headers = http::HeaderMap::new();
                let header_value = http::HeaderValue::from_str(&format!(
                    "{}={}",
                    session::SESSION_COOKIE_NAME,
                    cookie
                ))
                // SAFETY: It is known in advance that `SESSION_COOKIE_NAME` as
                // well as the cookie propagated from the init of the session
                // are both always going to be ASCII-only, therefore the
                // formatted string will be ASCII-only, so creating the
                // HeaderValue will never fail
                .unwrap();
                let _prev_value = headers.insert(http::header::SET_COOKIE, header_value);

                Ok((headers, http::StatusCode::NO_CONTENT))
            } else {
                Err(Error::WrongPassword)
            }
        }
        None => Err(Error::UserNotFound { username }),
    }
}

type Result<T> = ::core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub(crate) enum Error
{
    #[error("{0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("{0}")]
    Password(#[from] password::Error),
    #[error("{0}")]
    Session(#[from] session::Error),
    #[error("no user with username {username} was found")]
    UserNotFound
    {
        username: String
    },
    #[error("the provided password is wrong")]
    WrongPassword,
    #[error("must be authenticated")]
    MustBeAuthenticated,
}

impl response::IntoResponse for Error
{
    fn into_response(self) -> response::Response
    {
        match self {
            Error::UserNotFound { .. } => http::StatusCode::NOT_FOUND,
            Error::WrongPassword => http::StatusCode::UNPROCESSABLE_ENTITY,
            Error::MustBeAuthenticated => http::StatusCode::UNAUTHORIZED,
            _ => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
        .into_response()
    }
}
