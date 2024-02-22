use async_trait::async_trait;
use deadpool::managed;
use deadpool::managed::{Metrics, RecycleError};
use log::warn;

use crate::{Client, Error};

#[derive(Debug)]
pub struct Manager(String);
pub type Pool = managed::Pool<Manager>;

impl Manager {
    pub fn new<S: Into<String>>(url: S) -> Self {
        Self(url.into())
    }
}

#[async_trait]
impl managed::Manager for Manager {
    type Type = Client;
    type Error = Error;

    async fn create(&self) -> Result<Client, Error> {
        Ok(Client::connect(&self.0).await.unwrap())
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
