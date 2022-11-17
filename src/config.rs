use std::env;

pub use self::error::{Error, Result};

const FALLBACK_DATABASE_URL: &'static str = "postgres://postgres:postgres@localhost/bluebird";
const FALLBACK_PORT: u16 = 3000;

#[derive(Debug)]
pub struct Config
{
    database_url: String,
    port: u16,
}

impl Config
{
    pub fn init() -> Result<Self>
    {
        let database_url = match env::var("DATABASE_URL") {
            Ok(url) => url,
            Err(env::VarError::NotPresent) => String::from(FALLBACK_DATABASE_URL),
            err => err?,
        };

        let port = match env::var("PORT") {
            Ok(port) => port.parse()?,
            Err(env::VarError::NotPresent) => FALLBACK_PORT,
            Err(err) => Err(err)?,
        };

        Ok(Config { database_url, port })
    }

    pub fn database_url(&self) -> &str
    {
        &self.database_url
    }

    pub fn port(&self) -> u16
    {
        self.port
    }
}

pub mod error
{
    use std::{env, error, fmt, num, result};

    pub type Result<T> = result::Result<T, Error>;

    #[derive(Debug)]
    pub enum Error
    {
        EnvVar(env::VarError),
        ParseInt(num::ParseIntError),
    }

    impl fmt::Display for Error
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
        {
            match self {
                Error::EnvVar(env_var_err) => write!(f, "{env_var_err}"),
                Error::ParseInt(parse_int_err) => write!(f, "{parse_int_err}"),
            }
        }
    }

    impl error::Error for Error
    {
        fn source(&self) -> Option<&(dyn error::Error + 'static)>
        {
            match self {
                Error::EnvVar(env_var_err) => Some(env_var_err),
                Error::ParseInt(parse_int_err) => Some(parse_int_err),
            }
        }
    }

    impl From<env::VarError> for Error
    {
        fn from(env_var_err: env::VarError) -> Self
        {
            Error::EnvVar(env_var_err)
        }
    }

    impl From<num::ParseIntError> for Error
    {
        fn from(parse_int_err: num::ParseIntError) -> Self
        {
            Error::ParseInt(parse_int_err)
        }
    }
}
