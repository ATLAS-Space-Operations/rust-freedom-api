use freedom_config::Config;
use reqwest::{Response, StatusCode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use url::Url;

use crate::api::{FreedomApi, FreedomApiContainer, FreedomApiValue};

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
    /// + ATLAS_ENV: [possible values: local, dev, test, prod]
    /// + ATLAS_KEY: The ATLAS freedom key registered with an account
    /// + ATLAS_SECRET: The ATLAS freedom secret registered with an account
    pub fn from_env() -> Result<Self, freedom_config::Error> {
        let config = Config::from_env()?;
        Ok(Self::from_config(config))
    }

    pub(crate) async fn get_body(&self, url: Url) -> Result<String, crate::error::Error> {
        let resp = self
            .client
            .get(url.clone())
            .basic_auth(self.config.key(), Some(&self.config.expose_secret()))
            .send()
            .await;

        let resp = match resp {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!(error = ?e, "Received ERR response");
                return Err(From::from(e));
            }
        };
        tracing::debug!(?resp, "Received OK response");

        if resp.status() != StatusCode::OK {
            tracing::warn!(status = ?resp.status(), url = url.as_str(), "Received non-OK HTTP code");
        }
        let body = match resp.text().await {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!(error = ?e, "Received ERR body");
                return Err(From::from(e));
            }
        };
        tracing::debug!(?body, "Received OK body");

        Ok(body)
    }
}

/// A simple container which stores a `T`.
///
/// This container exists to allow us to store items on the stack, without needing to allocate with
/// something like `Box<T>`. For all other intents and purposes, it acts as the `T` which it
/// contains.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[repr(transparent)]
#[serde(transparent)]
pub struct Inner<T>(T);

impl<T> std::ops::Deref for Inner<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Inner<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> FreedomApiContainer<T> for Inner<T>
where
    T: FreedomApiValue,
{
    fn into_inner(self) -> T {
        self.0
    }
}

#[async_trait::async_trait]
impl FreedomApi for Client {
    type Container<T: FreedomApiValue> = Inner<T>;

    async fn get<T>(&self, url: Url) -> Result<T, crate::error::Error>
    where
        T: DeserializeOwned + Clone,
    {
        let body = self.get_body(url).await?;

        serde_json::from_str(&body).map_err(From::from)
    }

    async fn post<S, T>(&self, url: Url, msg: S) -> Result<T, crate::error::Error>
    async fn delete(&self, url: Url) -> Result<Response, crate::error::Error> {
        self.client
            .delete(url)
            .basic_auth(self.config.key(), Some(self.config.expose_secret()))
            .send()
            .await
            .map_err(From::from)
    }

    async fn post_response<S>(&self, url: Url, msg: S) -> Result<Response, crate::error::Error>
    where
        S: serde::Serialize + Sync + Send,
        T: DeserializeOwned + Clone,
    {
        let resp = self
            .client
            .post(url)
            .basic_auth(self.config.key(), Some(self.config.expose_secret()))
            .json(&msg)
            .send()
            .await?;

        resp.json::<T>().await.map_err(From::from)
    }

    fn config(&self) -> &Config {
        &self.config
    }

    fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }
}
