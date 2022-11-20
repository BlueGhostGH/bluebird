use std::{env, num};

use thiserror::Error;

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
    pub fn init() -> Result<Self, Error>
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

#[derive(Debug, Error)]
pub enum Error
{
    #[error("{0}")]
    EnvVar(#[from] env::VarError),
    #[error("{0}")]
    ParseInt(#[from] num::ParseIntError),
}
