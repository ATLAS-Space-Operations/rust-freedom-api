use bytes::Bytes;
use freedom_config::Config;
use reqwest::{Response, StatusCode};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::api::{Api, Container, Value};

/// An asynchronous `Client` for interfacing with the ATLAS freedom API.
///
/// The client is primarily defined based on it's [`AtlasEnv`](freedom_config::AtlasEnv)
/// and it's credentials (username and password).
#[derive(Clone, Debug)]
pub struct Client {
    pub(crate) config: Config,
    pub(crate) client: reqwest::Client,
}

impl PartialEq for Client {
    fn eq(&self, other: &Self) -> bool {
        self.config == other.config
    }
}

impl Client {
    pub fn from_config(config: Config) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// A convenience method for constructing an FPS client from environment variables.
    ///
    /// This function expects the following environment variables:
    ///
    /// + ATLAS_ENV: [possible values: test, prod]
    /// + ATLAS_KEY: The ATLAS freedom key registered with an account
    /// + ATLAS_SECRET: The ATLAS freedom secret registered with an account
    pub fn from_env() -> Result<Self, freedom_config::Error> {
        let config = Config::from_env()?;
        Ok(Self::from_config(config))
    }
}

/// A simple container which stores a `T`.
///
/// This container exists to allow us to store items on the stack, without needing to allocate with
/// something like `Box<T>`. For all other intents and purposes, it acts as the `T` which it
/// contains.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
#[serde(transparent)]
pub struct Inner<T>(T);

impl<T> std::ops::Deref for Inner<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Inner<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Container<T> for Inner<T>
where
    T: Value,
{
    fn into_inner(self) -> T {
        self.0
    }
}

impl Api for Client {
    type Container<T: Value> = Inner<T>;

    async fn get(&self, url: Url) -> Result<(Bytes, StatusCode), crate::error::Error> {
        let resp = self
            .client
            .get(url)
            .basic_auth(self.config.key(), Some(&self.config.expose_secret()))
            .send()
            .await?;

        let status = resp.status();
        let body = resp.bytes().await?;
        Ok((body, status))
    }

    async fn delete(&self, url: Url) -> Result<Response, crate::error::Error> {
        self.client
            .delete(url)
            .basic_auth(self.config.key(), Some(self.config.expose_secret()))
            .send()
            .await
            .map_err(From::from)
    }

    async fn post<S>(&self, url: Url, msg: S) -> Result<Response, crate::error::Error>
    where
        S: serde::Serialize + Sync + Send,
    {
        self.client
            .post(url)
            .basic_auth(self.config.key(), Some(self.config.expose_secret()))
            .json(&msg)
            .send()
            .await
            .map_err(From::from)
    }

    fn config(&self) -> &Config {
        &self.config
    }

    fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }
}
