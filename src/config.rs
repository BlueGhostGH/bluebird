use std::{env, num};

use thiserror::Error;

const FALLBACK_POSTGRES_URL: &str = "postgres://postgres:postgres@localhost/bluebird";
const FALLBACK_PORT: u16 = 3000;

#[derive(Debug)]
pub struct Config
{
    postgres_url: String,
    port: u16,
}

impl Config
{
    pub fn init() -> Result<Self>
    {
        let postgres_url = match env::var("POSTGRES_URL") {
            Ok(url) => url,
            Err(env::VarError::NotPresent) => String::from(FALLBACK_POSTGRES_URL),
            err => err?,
        };

        let port = match env::var("PORT") {
            Ok(port) => port.parse()?,
            Err(env::VarError::NotPresent) => FALLBACK_PORT,
            Err(err) => Err(err)?,
        };

        Ok(Config { postgres_url, port })
    }

    pub fn postgres_url(&self) -> &str
    {
        &self.postgres_url
    }

    pub fn port(&self) -> u16
    {
        self.port
    }
}

type Result<T> = ::core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error
{
    #[error("{0}")]
    EnvVar(#[from] env::VarError),
    #[error("{0}")]
    ParseInt(#[from] num::ParseIntError),
}
