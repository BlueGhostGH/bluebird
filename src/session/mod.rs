use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
};

use rand::RngCore;
use serde::Serialize;
use time::OffsetDateTime;

mod error;
pub use error::{Error, Result};

pub mod extractor;
pub mod store;

pub const SESSION_COOKIE_NAME: &str = "bluebird_session";

#[derive(Debug)]
pub struct Session
{
    id: String,
    expiry: Option<OffsetDateTime>,
    data: Arc<RwLock<HashMap<String, String>>>,

    cookie_value: Option<String>,
    data_changed: Arc<AtomicBool>,
}

pub fn generate_cookie(len: usize) -> String
{
    let mut key = vec![0u8; len];
    rand::thread_rng().fill_bytes(&mut key);
    base64::encode(key)
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

    pub fn get<T>(&self, key: &str) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let data = self.data.read().unwrap();
        let string = data.get(key)?;
        serde_json::from_str(string).ok()
    }

    pub fn insert<V>(&mut self, key: &str, value: V) -> Result<()>
    where
        V: Serialize,
    {
        self.insert_raw(key, serde_json::to_string(&value)?);
        Ok(())
    }

    fn insert_raw(&mut self, key: &str, value: String)
    {
        let mut data = self.data.write().unwrap();
        if data.get(key) != Some(&value) {
            data.insert(String::from(key), value);
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
            true => Err(Error::SessionExpired(
                // SAFETY: The only way `self.is_expired` could return true
                // is if, above everything else, the `self.expiry` field is of
                // the `Some `variant, therefore it is also guaranteed to be
                // `Some` here
                self.expiry.unwrap() - OffsetDateTime::now_utc(),
            )),
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
