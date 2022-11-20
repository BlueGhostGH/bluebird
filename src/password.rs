use axum::{http, response};
use tokio::task;

use argon2::{
    password_hash::{self, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use thiserror::Error;

pub async fn hash(password: String) -> Result<String>
{
    let password = task::spawn_blocking(move || {
        let salt = SaltString::generate(rand::thread_rng());

        let hashed_password = Argon2::default().hash_password(password.as_bytes(), &salt)?;

        Ok(hashed_password.to_string())
    })
    .await?;

    password
}

pub async fn verify(password: String, hash: String) -> Result<bool>
{
    task::spawn_blocking(move || {
        let hash = PasswordHash::new(&hash).map_err(Error::from)?;
        let is_correct = Argon2::default().verify_password(password.as_bytes(), &hash);

        match is_correct {
            Ok(()) => Ok(true),
            Err(password_hash::Error::Password) => Ok(false),
            Err(err) => Err(err)?,
        }
    })
    .await?
}

type Result<T> = ::core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error
{
    #[error("{0}")]
    PasswordHash(#[from] password_hash::Error),
    #[error("{0}")]
    TaskJoin(#[from] task::JoinError),
}

impl response::IntoResponse for Error
{
    fn into_response(self) -> response::Response
    {
        http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}
