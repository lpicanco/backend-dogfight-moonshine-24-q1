use async_trait::async_trait;
use deadpool::{managed};
use deadpool::managed::{Metrics, RecycleError};
use log::warn;

use crate::{Client, Error};

#[derive(Debug)]
pub struct Manager(String);
pub type Pool = managed::Pool<Manager>;

impl Manager {
    /// Creates a new [`Manager`] using the given [`Config`] backed by the
    /// specified [`Runtime`].
    pub fn new<S: Into<String>>(url: S) -> Self {
        Self(url.into())
    }
}

#[async_trait]
impl managed::Manager for Manager {
    type Type = Client;
    type Error = Error;

    async fn create(&self) -> Result<Client, Error> {
        // let (conn, ldap) = LdapConnAsync::with_settings(self.1.clone(), &self.0).await?;
        #[cfg(feature = "default")]
        ldap3::drive!(conn);
        #[cfg(feature = "rt-actix")]
        actix_rt::spawn(async move {
            if let Err(e) = conn.drive().await {
                log::warn!("LDAP connection error: {:?}", e);
            }
        });
        // Ok(ldap)

        warn!("Creating new client");

        let client = Client::connect(&self.0).await.unwrap();
        Ok(client)
    }

    async fn recycle(
        &self,
        conn: &mut Self::Type,
        _: &Metrics,
    ) -> managed::RecycleResult<Self::Error> {

        if conn.is_closed() {
            warn!("Recycling client");
            return Err(RecycleError::Message(
                "Connection is closed. Connection is considered unusable.".into(),
            ));
        }

        Ok(())
    }
}
