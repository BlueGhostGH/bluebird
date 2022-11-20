use std::time::Duration;

use sqlx::postgres::PgPoolOptions;

use bluebird::{config::Config, session::store::Store};

use error::Result;

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

    let store = Store::new();

    bluebird::http::serve(config.port(), pool, store).await?;

    Ok(())
}

mod error
{
    use std::{error, fmt, result};

    use bluebird::{config, http};

    pub type Result<T> = result::Result<T, Error>;

    #[derive(Debug)]
    pub enum Error
    {
        Config(config::Error),
        Sqlx(sqlx::Error),
        Migrate(sqlx::migrate::MigrateError),
        Hyper(hyper::Error),
        Http(http::Error),
    }

    impl fmt::Display for Error
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
        {
            match self {
                Error::Config(config_err) => write!(f, "{config_err}"),
                Error::Sqlx(sqlx_err) => write!(f, "{sqlx_err}"),
                Error::Migrate(migrate_err) => write!(f, "{migrate_err}"),
                Error::Hyper(hyper_err) => write!(f, "{hyper_err}"),
                Error::Http(http_err) => write!(f, "{http_err}"),
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
                Error::Migrate(migrate_err) => Some(migrate_err),
                Error::Hyper(hyper_err) => Some(hyper_err),
                Error::Http(http_err) => Some(http_err),
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

    impl From<sqlx::migrate::MigrateError> for Error
    {
        fn from(migrate_err: sqlx::migrate::MigrateError) -> Self
        {
            Error::Migrate(migrate_err)
        }
    }

    impl From<hyper::Error> for Error
    {
        fn from(hyper_err: hyper::Error) -> Self
        {
            Error::Hyper(hyper_err)
        }
    }

    impl From<http::Error> for Error
    {
        fn from(http_err: http::Error) -> Self
        {
            Error::Http(http_err)
        }
    }
}
