use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
    time::Duration,
};

use axum::{http, response};

use rand::RngCore;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod extractor;
mod store;

pub use store::Store;

pub const SESSION_COOKIE_NAME: &str = "bluebird_session";

pub fn generate_cookie(len: usize) -> String
{
    let mut key = vec![0u8; len];
    rand::thread_rng().fill_bytes(&mut key);
    base64::encode(key)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Session
{
    id: String,
    expires_in: Option<Duration>,
    data: Arc<RwLock<HashMap<String, String>>>,

    #[serde(skip)]
    cookie_value: Option<String>,
    #[serde(skip)]
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
            expires_in: None,
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
        // TODO: Figure out how ok it is to unwrap here
        let data = self.data.read().unwrap();
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
        // TODO: Same as line 75
        let mut data = self.data.write().unwrap();
        if data.get(key) != Some(&value) {
            let _prev_val = data.insert(String::from(key), value);
            self.data_changed.store(true, Ordering::Relaxed);
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
            expires_in: self.expires_in,
            data: self.data.clone(),

            cookie_value: None,
            data_changed: self.data_changed.clone(),
        }
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
    #[error("{0}")]
    Redis(#[from] redis::RedisError),
    #[error("missing request session store extension")]
    MissingStoreExtension,
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
            | Error::Redis(_)
            | Error::MissingStoreExtension => {
                http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            Error::NoSessionFound { .. } => http::StatusCode::BAD_REQUEST.into_response(),
        }
    }
}
