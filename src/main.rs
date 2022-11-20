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

    let pg_pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(config.postgres_url())
        .await?;
    sqlx::migrate!().run(&pg_pool).await?;

    let redis_client = redis::Client::open("redis://127.0.0.1/")?;
    let session_store = session::Store::new(redis_client);

    bluebird::http::serve(config.port(), pg_pool, session_store).await?;

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
    Redis(#[from] redis::RedisError),
    #[error("{0}")]
    Hyper(#[from] hyper::Error),
    #[error("{0}")]
    Http(#[from] http::Error),
}
