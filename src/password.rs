use tokio::task;

use argon2::{password_hash::SaltString, Argon2, PasswordHasher};

pub use error::{Error, Result};

pub async fn hash(password: String) -> Result<String>
{
    let password = task::spawn_blocking(move || {
        let salt = SaltString::generate(rand::thread_rng());

        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|pw| pw.to_string())
            .map_err(error::Error::from)
    })
    .await?;

    password
}

mod error
{
    use std::{error, fmt, result};

    use axum::{http, response};
    use tokio::task;

    use argon2::password_hash;

    pub type Result<T> = result::Result<T, Error>;

    #[derive(Debug)]
    pub enum Error
    {
        PasswordHash(password_hash::Error),
        TaskJoin(task::JoinError),
    }

    impl fmt::Display for Error
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
        {
            match self {
                Error::PasswordHash(password_hash_err) => write!(f, "{password_hash_err}"),
                Error::TaskJoin(task_join_err) => write!(f, "{task_join_err}"),
            }
        }
    }

    impl error::Error for Error
    {
        fn source(&self) -> Option<&(dyn error::Error + 'static)>
        {
            match self {
                Error::PasswordHash(password_hash_err) => Some(password_hash_err),
                Error::TaskJoin(task_join_err) => Some(task_join_err),
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

    impl From<password_hash::Error> for Error
    {
        fn from(password_hash_err: password_hash::Error) -> Self
        {
            Error::PasswordHash(password_hash_err)
        }
    }

    impl From<task::JoinError> for Error
    {
        fn from(task_join_err: task::JoinError) -> Self
        {
            Error::TaskJoin(task_join_err)
        }
    }
}
