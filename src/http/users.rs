use axum::{http, response, routing::post, Extension, Json, Router};
use sqlx::PgPool;

use serde::Deserialize;
use thiserror::Error;

use crate::password;

pub(crate) fn router() -> Router
{
    Router::new().route("/users", post(create_user))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateUser
{
    username: String,
    password: String,
}

async fn create_user(
    pg_pool: Extension<PgPool>,
    Json(req): Json<CreateUser>,
) -> Result<http::StatusCode>
{
    let CreateUser { username, password } = req;

    let password = password::hash(password).await?;

    let _pg_query_res = sqlx::query!(
        r#"
            INSERT INTO "users"(username, password)
            values ($1, $2)
        "#,
        username,
        password
    )
    .execute(&*pg_pool)
    .await
    .map_err(|err| match err {
        sqlx::Error::Database(db_err) if db_err.constraint() == Some("users_username_key") => {
            Error::UsernameTaken
        }
        err => err.into(),
    })?;

    Ok(http::StatusCode::NO_CONTENT)
}

type Result<T> = ::core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub(crate) enum Error
{
    #[error("{0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("{0}")]
    Password(#[from] password::Error),
    #[error("username already taken")]
    UsernameTaken,
}

impl response::IntoResponse for Error
{
    fn into_response(self) -> response::Response
    {
        match self {
            Error::UsernameTaken => http::StatusCode::CONFLICT,
            _ => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
        .into_response()
    }
}
