use std::{collections::HashMap, sync::Arc};

use async_lock::RwLock;

use crate::session::{Error, Session};

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

    pub async fn load_session(&self, cookie: &str) -> Result<Session, Error>
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

    pub async fn store_session(&self, session: Session) -> Result<Option<String>, Error>
    {
        self.inner
            .write()
            .await
            .insert(session.id.clone(), session.clone());

        session.reset_data_changed();
        Ok(session.into_cookie_value())
    }
}
