use redis::AsyncCommands;

use crate::http::session;

#[derive(Debug, Clone)]
pub struct Store
{
    client: redis::Client,
}

impl Store
{
    pub fn new(client: redis::Client) -> Self
    {
        Store { client }
    }

    async fn connection(&self) -> session::Result<redis::aio::Connection>
    {
        self.client
            .get_tokio_connection()
            .await
            .map_err(session::Error::from)
    }

    pub(super) async fn load_session(&self, cookie: &str) -> session::Result<session::Session>
    {
        let id = session::Session::id_from_cookie(cookie)?;
        let mut connection = self.connection().await?;

        let record = connection
            .get::<_, Option<String>>(id)
            .await
            .map(|rec| {
                rec.ok_or_else(|| {
                    let cookie = String::from(cookie);
                    session::Error::NoSessionFound { cookie }
                })
            })
            .map_err(session::Error::from)
            .flatten()?;

        let session = serde_json::from_str(&record)?;

        Ok(session)
    }

    pub(in crate::http) async fn store_session(
        &self,
        session: session::Session,
    ) -> session::Result<Option<String>>
    {
        let record = serde_json::to_string(&session)?;
        let mut connection = self.connection().await?;

        match session.expires_in {
            Some(expiry) => {
                connection
                    .set_ex(session.id.clone(), record, expiry.as_secs() as usize)
                    .await?
            }
            None => connection.set(session.id.clone(), record).await?,
        };

        Ok(session.into_cookie_value())
    }
}
