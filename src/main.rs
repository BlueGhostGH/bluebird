use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use thiserror::Error;

use bluebird::{
    config::{self, Config},
    http::{self, session},
};

#[tokio::main]
async fn main() -> Result<()>
{
    tracing_subscriber::fmt::init();

    let config = Config::init()?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(config.database_url())
        .await?;
    sqlx::migrate!().run(&pool).await?;

    let session_store = session::Store::new();

    bluebird::http::serve(config.port(), pool, session_store).await?;

    Ok(())
}

type Result<T> = ::core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error
{
    #[error("{0}")]
    Config(#[from] config::Error),
    #[error("{0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("{0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("{0}")]
    Hyper(#[from] hyper::Error),
    #[error("{0}")]
    Http(#[from] http::Error),
}
