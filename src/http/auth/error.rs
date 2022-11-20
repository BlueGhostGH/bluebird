use std::{error, fmt, result};

use axum::{http, response};

use crate::{password, session};

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error
{
    Sqlx(sqlx::Error),
    Password(password::Error),
    Session(session::Error),
    UserNotFound(String),
    WrongPassword,
    MustBeAuthenticated,
}

impl fmt::Display for Error
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self {
            Error::Sqlx(sqlx_err) => write!(f, "{sqlx_err}"),
            Error::Password(password_err) => write!(f, "{password_err}"),
            Error::Session(session_err) => write!(f, "{session_err}"),
            Error::UserNotFound(username) => {
                write!(f, "no user with username {username} was found")
            }
            Error::WrongPassword => write!(f, "the provided password is wrong"),
            Error::MustBeAuthenticated => write!(f, "must be authenticated"),
        }
    }
}

impl error::Error for Error
{
    fn source(&self) -> Option<&(dyn error::Error + 'static)>
    {
        match self {
            Error::Sqlx(sqlx_err) => Some(sqlx_err),
            Error::Password(password_err) => Some(password_err),
            Error::Session(session_err) => Some(session_err),
            _ => None,
        }
    }
}

impl response::IntoResponse for Error
{
    fn into_response(self) -> response::Response
    {
        match self {
            Error::UserNotFound(_) => http::StatusCode::NOT_FOUND,
            Error::WrongPassword => http::StatusCode::UNPROCESSABLE_ENTITY,
            Error::MustBeAuthenticated => http::StatusCode::UNAUTHORIZED,
            _ => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
        .into_response()
    }
}

impl From<sqlx::Error> for Error
{
    fn from(sqlx_err: sqlx::Error) -> Self
    {
        Error::Sqlx(sqlx_err)
    }
}

impl From<password::Error> for Error
{
    fn from(password_err: password::Error) -> Self
    {
        Error::Password(password_err)
    }
}

impl From<session::Error> for Error
{
    fn from(session_err: session::Error) -> Self
    {
        Error::Session(session_err)
    }
}
