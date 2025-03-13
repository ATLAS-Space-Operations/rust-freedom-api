use std::sync::Arc;

use bytes::Bytes;
use freedom_config::Config;
use moka::future::Cache;
use reqwest::{Response, StatusCode};
use url::Url;

use crate::{
    api::{Api, Container, Value},
    error::Error,
    Client,
};

/// An asynchronous `Client` for interfacing with the ATLAS freedom API, which implements query
/// caching.
///
/// This client has the same API as the normal [`Client`](crate::client::Client), however queries
/// and their associated responses are cached before being delivered.
///
/// As a result, the items which are returned to the caller are wrapped in [`Arc`](std::sync::Arc).
/// This makes cloning items out of the cache extremely cheap, regardless of the object's actual
/// size.
#[derive(Clone, Debug)]
pub struct CachingClient {
    pub(crate) inner: Client,
    pub(crate) cache: Cache<Url, (Bytes, StatusCode)>,
}

impl CachingClient {
    /// Create a new caching client from a normal client
    pub fn new(client: Client, capacity: u64) -> Self {
        Self {
            inner: client,
            cache: Cache::new(capacity),
        }
    }

    /// Invalidates all the entries in the cache. Future requests to `get` will result in new calls
    /// to Freedom
    pub fn invalidate_all(&self) {
        self.cache.invalidate_all();
    }
}

impl PartialEq for CachingClient {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T: Value> Container<T> for Arc<T> {
    fn into_inner(self) -> T {
        std::sync::Arc::<T>::unwrap_or_clone(self)
    }
}

impl Api for CachingClient {
    type Container<T: Value> = Arc<T>;

    async fn delete(&self, url: Url) -> Result<Response, Error> {
        self.inner.delete(url).await
    }

    /// # Panics
    ///
    /// Panics if called outside of a tokio runtime
    async fn get(&self, url: Url) -> Result<(Bytes, StatusCode), Error> {
        let client = self.inner.clone();
        let url_clone = url.clone();

        let fut = async move {
            let (body, status) = client.get(url_clone).await?;

            Ok::<_, Error>((body, status))
        };

        let cache = self.cache.clone();

        tokio::spawn(async move {
            cache
                .try_get_with(url, fut)
                .await
                .map_err(Arc::unwrap_or_clone)
        })
        .await
        .map_err(|error| Error::Response(error.to_string()))?
    }

    async fn post<S>(&self, url: Url, msg: S) -> Result<Response, Error>
    where
        S: serde::Serialize + Send + Sync,
    {
        self.inner.post(url, msg).await
    }

    fn config(&self) -> &Config {
        self.inner.config()
    }

    fn config_mut(&mut self) -> &mut Config {
        self.inner.config_mut()
    }
}
