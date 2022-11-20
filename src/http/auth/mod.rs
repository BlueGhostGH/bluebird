use std::ops::Deref;

use axum::{http, routing::get, Extension, Json, Router};
use sqlx::PgPool;

use serde::Deserialize;

use crate::{
    password,
    session::{self, extractor, store::Store, Session},
};

mod error;
pub use error::{Error, Result};

pub fn router() -> Router
{
    Router::new().route("/auth", get(fetch_auth_session).post(create_auth_session))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAuthSession
{
    username: String,
    password: String,
}

async fn fetch_auth_session(user_id: extractor::UserId) -> Result<String>
{
    match user_id {
        extractor::UserId::Found(user_id) => Ok(user_id.to_string()),
        extractor::UserId::NotFound => Err(Error::MustBeAuthenticated),
    }
}

async fn create_auth_session(
    db_pool: Extension<PgPool>,
    store: Extension<Store>,
    Json(req): Json<CreateAuthSession>,
) -> Result<(http::HeaderMap, http::StatusCode)>
{
    let CreateAuthSession { username, password } = req;

    let user = sqlx::query!(
        r#"select user_id, password from users where username = $1"#,
        username
    )
    .fetch_optional(db_pool.deref())
    .await?;

    match user {
        Some(user) => {
            let password_is_correct = password::verify(password, user.password).await?;

            if password_is_correct {
                let mut session = Session::new();
                session.insert("user_id", user.user_id)?;
                // SAFETY: This cannot fail as store_session propagates `None`
                // upon a `None` field for the session's cookie value, which
                // will never be empty as we create the session above and never
                // mutate its cookie value
                let cookie = store.store_session(session).await?.unwrap();

                let mut headers = http::HeaderMap::new();
                let header_value = http::HeaderValue::from_str(&format!(
                    "{}={}",
                    session::SESSION_COOKIE_NAME,
                    cookie
                ))
                // SAFETY: It is known in advance that `SESSION_COOKIE_NAME` as
                // well as the cookie propagated from the init of the session
                // are both always going to be ASCII-only, therefore the
                // formatted string will be ASCII-only, so creating the
                // HeaderValue will never fail
                .unwrap();
                headers.insert(http::header::SET_COOKIE, header_value);

                Ok((headers, http::StatusCode::NO_CONTENT))
            } else {
                Err(error::Error::WrongPassword)
            }
        }
        None => Err(error::Error::UserNotFound(username)),
    }
}
