use std::{error, fmt, result};

use crate::config;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error
{
    Config(config::Error),
    Sqlx(sqlx::Error),
    Hyper(hyper::Error),
}

impl fmt::Display for Error
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self {
            Error::Config(config_err) => write!(f, "{config_err}"),
            Error::Sqlx(sqlx_err) => write!(f, "{sqlx_err}"),
            Error::Hyper(hyper_err) => write!(f, "{hyper_err}"),
        }
    }
}

impl error::Error for Error
{
    fn source(&self) -> Option<&(dyn error::Error + 'static)>
    {
        match self {
            Error::Config(config_err) => Some(config_err),
            Error::Sqlx(sqlx_err) => Some(sqlx_err),
            Error::Hyper(hyper_err) => Some(hyper_err),
        }
    }
}

impl From<config::Error> for Error
{
    fn from(config_err: config::Error) -> Self
    {
        Error::Config(config_err)
    }
}

impl From<sqlx::Error> for Error
{
    fn from(sqlx_err: sqlx::Error) -> Self
    {
        Error::Sqlx(sqlx_err)
    }
}

impl From<hyper::Error> for Error
{
    fn from(hyper_err: hyper::Error) -> Self
    {
        Error::Hyper(hyper_err)
    }
}
