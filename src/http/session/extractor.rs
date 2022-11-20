use axum::{
    extract::{FromRequestParts, TypedHeader},
    headers::Cookie,
    http, Extension, RequestPartsExt,
};

use async_trait::async_trait;
use uuid::Uuid;

use crate::http::session;

#[derive(Debug, Clone, Copy)]
pub enum UserId
{
    Found(Uuid),
    NotFound,
}

#[async_trait]
impl<S> FromRequestParts<S> for UserId
where
    S: Send + Sync,
{
    type Rejection = session::Error;

    async fn from_request_parts(
        parts: &mut http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection>
    {
        let store = parts
            .extract::<Extension<session::Store>>()
            .await
            .map_err(|_| session::Error::MissingStoreExtension)?;
        let cookie = parts
            .extract::<Option<TypedHeader<Cookie>>>()
            .await
            // SAFETY: Unwrapping `Result<T, Infallible>` is guaranteed to
            // never panic
            .unwrap();
        let session_cookie = cookie
            .as_ref()
            .and_then(|cookie| cookie.get(session::SESSION_COOKIE_NAME));

        match session_cookie {
            Some(session_cookie) => {
                let session = store.load_session(session_cookie).await?;

                if let Some(user_id) = session.get::<Uuid>("user_id").await {
                    Ok(UserId::Found(user_id))
                } else {
                    Ok(UserId::NotFound)
                }
            }
            None => Ok(UserId::NotFound),
        }
    }
}
