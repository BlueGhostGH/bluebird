use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use axum::{http, response};

use async_lock::RwLock;
use rand::RngCore;
use serde::Serialize;
use thiserror::Error;
use time::{Duration, OffsetDateTime};

pub mod extractor;

pub const SESSION_COOKIE_NAME: &str = "bluebird_session";

pub fn generate_cookie(len: usize) -> String
{
    let mut key = vec![0u8; len];
    rand::thread_rng().fill_bytes(&mut key);
    base64::encode(key)
}

#[derive(Debug)]
pub struct Session
{
    id: String,
    expiry: Option<OffsetDateTime>,
    data: Arc<RwLock<HashMap<String, String>>>,

    cookie_value: Option<String>,
    data_changed: Arc<AtomicBool>,
}

impl Session
{
    pub fn new() -> Self
    {
        let cookie = generate_cookie(64);
        // SAFETY: This cannot fail as the cookie is not mutated between the
        // base64 encoding and the base64 decoding, which is the only step at
        // which the below call could fail
        let id = Session::id_from_cookie(&cookie).unwrap();

        Session {
            id,
            expiry: None,
            data: Arc::new(RwLock::new(HashMap::default())),

            cookie_value: Some(cookie),
            data_changed: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn id_from_cookie(cookie: &str) -> Result<String>
    {
        let decoded = base64::decode(cookie)?;
        let hash = blake3::hash(&decoded);

        Ok(base64::encode(hash.as_bytes()))
    }

    pub async fn get<T>(&self, key: &str) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let data = self.data.read().await;
        let string = data.get(key)?;
        serde_json::from_str(string).ok()
    }

    pub async fn insert<V>(&mut self, key: &str, value: V) -> Result<()>
    where
        V: Serialize,
    {
        self.insert_raw(key, serde_json::to_string(&value)?).await;
        Ok(())
    }

    async fn insert_raw(&mut self, key: &str, value: String)
    {
        let mut data = self.data.write().await;
        if data.get(key) != Some(&value) {
            let _prev_val = data.insert(String::from(key), value);
            self.data_changed.store(true, Ordering::Relaxed);
        }
    }

    pub fn is_expired(&self) -> bool
    {
        match self.expiry {
            Some(expiry) => expiry < OffsetDateTime::now_utc(),
            None => false,
        }
    }

    pub fn validate(self) -> Result<Self>
    {
        match self.is_expired() {
            false => Ok(self),
            true => {
                // SAFETY: The only way `self.is_expired` could return true
                // is if, above everything else, the `self.expiry` field is of
                // the `Some `variant, therefore it is also guaranteed to be
                // `Some` here
                let by = self.expiry.unwrap() - OffsetDateTime::now_utc();

                Err(Error::SessionExpired { by })
            }
        }
    }

    pub fn reset_data_changed(&self)
    {
        self.data_changed.store(false, Ordering::Relaxed);
    }

    pub fn into_cookie_value(mut self) -> Option<String>
    {
        self.cookie_value.take()
    }
}

impl Clone for Session
{
    fn clone(&self) -> Self
    {
        Session {
            id: self.id.clone(),
            expiry: self.expiry,
            data: self.data.clone(),

            cookie_value: None,
            data_changed: self.data_changed.clone(),
        }
    }
}

// TODO: move to a Redis-based store
#[derive(Debug, Clone)]
pub struct Store
{
    inner: Arc<RwLock<HashMap<String, Session>>>,
}

impl Store
{
    pub fn new() -> Self
    {
        Store {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn load_session(&self, cookie: &str) -> Result<Session>
    {
        let id = Session::id_from_cookie(cookie)?;
        let store = self.inner.read().await;
        let session = store
            .get(&id)
            .cloned()
            .ok_or_else(|| {
                let cookie = String::from(cookie);
                Error::NoSessionFound { cookie }
            })
            .and_then(Session::validate);

        session
    }

    pub async fn store_session(&self, session: Session) -> Result<Option<String>>
    {
        let _prev_val = self
            .inner
            .write()
            .await
            .insert(session.id.clone(), session.clone());

        session.reset_data_changed();
        Ok(session.into_cookie_value())
    }
}

type Result<T> = ::core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error
{
    #[error("{0}")]
    Base64Decode(#[from] base64::DecodeError),
    #[error("{0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("missing request session store extension")]
    MissingStoreExtension,
    #[error("session has been expired for {by}")]
    SessionExpired
    {
        by: Duration
    },
    #[error("no session found for cookie {cookie}")]
    NoSessionFound
    {
        cookie: String
    },
}

impl response::IntoResponse for Error
{
    fn into_response(self) -> response::Response
    {
        match self {
            Error::Base64Decode(_)
            | Error::SerdeJson(_)
            | Error::MissingStoreExtension
            | Error::SessionExpired { .. } => {
                http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            Error::NoSessionFound { .. } => http::StatusCode::BAD_REQUEST.into_response(),
        }
    }
}
