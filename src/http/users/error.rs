use std::{error, fmt, result};

use axum::{http, response};

use crate::password;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error
{
    Sqlx(sqlx::Error),
    Password(password::Error),
    UsernameTaken,
}

impl fmt::Display for Error
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self {
            Error::Sqlx(sqlx_err) => write!(f, "{sqlx_err}"),
            Error::Password(password_err) => write!(f, "{password_err}"),
            Error::UsernameTaken => write!(f, "username already taken"),
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
            _ => None,
        }
    }
}

impl response::IntoResponse for Error
{
    fn into_response(self) -> response::Response
    {
        match self {
            Error::UsernameTaken => http::StatusCode::CONFLICT,
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
