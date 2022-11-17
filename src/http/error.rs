use std::{error, fmt, result};

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error
{
    Hyper(hyper::Error),
}

impl fmt::Display for Error
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self {
            Error::Hyper(hyper_err) => write!(f, "{hyper_err}"),
        }
    }
}

impl error::Error for Error
{
    fn source(&self) -> Option<&(dyn error::Error + 'static)>
    {
        match self {
            Error::Hyper(hyper_err) => Some(hyper_err),
        }
    }
}

impl From<hyper::Error> for Error
{
    fn from(hyper_err: hyper::Error) -> Self
    {
        Error::Hyper(hyper_err)
    }
}
