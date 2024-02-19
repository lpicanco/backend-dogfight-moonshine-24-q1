use std::convert::Infallible;

use async_trait::async_trait;
use deadpool::{managed};
use deadpool::managed::{Metrics, RecycleError};
use log::warn;

use crate::{Client, Error};

/*deadpool::managed_reexports!(
    "rusqlite",
    Manager,
    deadpool::managed::Object<Manager>,
    // rusqlite::Error,
    Error,
    ConfigError
);*/

/*#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde_1::Deserialize, serde_1::Serialize))]
#[cfg_attr(feature = "serde", serde(crate = "serde_1"))]
pub struct Config {
    /// Path to SQLite database file.
    pub url: String,

    /// [`Pool`] configuration.
    pub pool: Option<PoolConfig>,
}

// pub enum Error { Fail }
// type Pool = managed::Pool<Manager>;
// type PoolBuilder = managed::PoolBuilder<Manager>;

impl Config {
    /// Create a new [`Config`] with the given `path` of SQLite database file.
    #[must_use]
    pub fn new(url: String) -> Self {
        Self {
            url,
            pool: None,
        }
    }

    /// Creates a new [`Pool`] using this [`Config`].
    ///
    /// # Errors
    ///
    /// See [`CreatePoolError`] for details.
    ///
    /// [`RedisError`]: redis::RedisError
    pub fn create_pool(&self, runtime: Runtime) -> Result<Pool, CreatePoolError> {
        self.builder(runtime)
            .map_err(CreatePoolError::Config)?
            .runtime(runtime)
            .build()
            .map_err(CreatePoolError::Build)
    }
    /// Creates a new [`PoolBuilder`] using this [`Config`].
    ///
    /// # Errors
    ///
    /// See [`ConfigError`] for details.
    ///
    /// [`RedisError`]: redis::RedisError
    pub fn builder(&self, runtime: Runtime) -> Result<PoolBuilder, ConfigError> {
        let manager = Manager::from_config(self, runtime);
        Ok(Pool::builder(manager)
            .config(self.get_pool_config())
            .runtime(runtime))
    }

    /// Returns [`deadpool::managed::PoolConfig`] which can be used to construct
    /// a [`deadpool::managed::Pool`] instance.
    #[must_use]
    pub fn get_pool_config(&self) -> PoolConfig {
        self.pool.unwrap_or_default()
    }
}*/

#[derive(Debug)]
pub struct Manager(String);
pub type Pool = managed::Pool<Manager>;
// #[derive(Debug)]
// pub struct Manager {
//     config: Config,
//     recycle_count: AtomicUsize,
//     runtime: Runtime,
// }

impl Manager {
    /// Creates a new [`Manager`] using the given [`Config`] backed by the
    /// specified [`Runtime`].
    pub fn new<S: Into<String>>(url: S) -> Self {
        Self(url.into())
    }
/*    #[must_use]
    pub fn from_config(config: &Config, runtime: Runtime) -> Self {
        Self {
            config: config.clone(),
            recycle_count: AtomicUsize::new(0),
            runtime,
        }
    }*/
}

#[async_trait]
impl managed::Manager for Manager {
    type Type = Client;
    type Error = Error;
    // type Type = SyncWrapper<rusqlite::Connection>;
    // type Error = rusqlite::Error;

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

/*    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let url = self.config.url.clone();
        Ok(Client::connect(url).await.unwrap())
    }
*/
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

pub type ConfigError = Infallible;