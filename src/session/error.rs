use std::{error, fmt, result};

use axum::{http, response};
use time::Duration;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error
{
    Base64Decode(base64::DecodeError),
    SerdeJson(serde_json::Error),
    MissingStoreExtension,
    SessionExpired(Duration),
    NoSessionFound(String),
}

impl fmt::Display for Error
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self {
            Error::Base64Decode(base64_decode_err) => write!(f, "{base64_decode_err}"),
            Error::SerdeJson(serde_json_err) => write!(f, "{serde_json_err}"),
            Error::MissingStoreExtension => write!(f, "missing request session store extension"),
            Error::SessionExpired(by) => write!(f, "session has already expired for {by}"),
            Error::NoSessionFound(cookie) => write!(f, "no session found for cookie {cookie}"),
        }
    }
}

impl error::Error for Error
{
    fn source(&self) -> Option<&(dyn error::Error + 'static)>
    {
        match self {
            Error::Base64Decode(base64_decode_err) => Some(base64_decode_err),
            Error::SerdeJson(serde_json_err) => Some(serde_json_err),
            _ => None,
        }
    }
}

impl response::IntoResponse for Error
{
    fn into_response(self) -> response::Response
    {
        match self {
            Error::Base64Decode(_)
            | Error::SerdeJson(_)
            | Error::MissingStoreExtension
            | Error::SessionExpired(_) => http::StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            Error::NoSessionFound(_) => http::StatusCode::BAD_REQUEST.into_response(),
        }
    }
}

impl From<base64::DecodeError> for Error
{
    fn from(base64_decode_err: base64::DecodeError) -> Self
    {
        Error::Base64Decode(base64_decode_err)
    }
}

impl From<serde_json::Error> for Error
{
    fn from(serde_json_err: serde_json::Error) -> Self
    {
        Error::SerdeJson(serde_json_err)
    }
}
