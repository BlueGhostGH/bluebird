use axum::extract::State;
use sqlx::postgres::PgPool;

pub async fn root(State(pool): State<PgPool>) -> error::Result<String>
{
    let (val,): (i64,) = sqlx::query_as("SELECT $1")
        .bind(256_i64)
        .fetch_one(&pool)
        .await?;

    Ok(val.to_string())
}

mod error
{
    use std::{error, fmt, result};

    use axum::{http, response};

    pub type Result<T> = result::Result<T, Error>;

    #[derive(Debug)]
    pub enum Error
    {
        Sqlx(sqlx::Error),
    }

    impl fmt::Display for Error
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
        {
            match self {
                Error::Sqlx(sqlx_err) => write!(f, "{sqlx_err}"),
            }
        }
    }

    impl error::Error for Error
    {
        fn source(&self) -> Option<&(dyn error::Error + 'static)>
        {
            match self {
                Error::Sqlx(sqlx_err) => Some(sqlx_err),
            }
        }
    }

    impl response::IntoResponse for Error
    {
        fn into_response(self) -> response::Response
        {
            http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }

    impl From<sqlx::Error> for Error
    {
        fn from(sqlx_err: sqlx::Error) -> Self
        {
            Error::Sqlx(sqlx_err)
        }
    }
}
