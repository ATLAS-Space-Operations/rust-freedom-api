use std::sync::Arc;

use bytes::Bytes;
use freedom_config::Config;
use reqwest::{Response, StatusCode};
use url::Url;

use crate::{
    api::{FreedomApi, FreedomApiContainer, FreedomApiValue},
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
    pub(crate) cache: moka::future::Cache<Url, (Bytes, StatusCode)>,
}

impl PartialEq for CachingClient {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T: FreedomApiValue> FreedomApiContainer<T> for Arc<T> {
    fn into_inner(self) -> T {
        std::sync::Arc::<T>::unwrap_or_clone(self)
    }
}

#[async_trait::async_trait]
impl FreedomApi for CachingClient {
    type Container<T: FreedomApiValue> = Arc<T>;

    async fn delete(&self, url: Url) -> Result<Response, crate::error::Error> {
        self.inner.delete(url).await
    }

    #[tracing::instrument]
    async fn get(&self, url: Url) -> Result<(Bytes, StatusCode), Error> {
        use crate::error::RuntimeError;

        // This is a rather cheap clone. Something like 50 bytes. This is necessary since we will
        // be passing this to the tokio executor which has lifetime requirements of `'static`
        let client = &self.inner;
        let value = self
            .cache
            .try_get_with(url.clone(), async {
                let (body, status) = client.get(url).await?;

                if !status.is_success() {
                    return Err(Error::Runtime(RuntimeError::Response(status.to_string())));
                }

                Ok((body, status))
            })
            .await;

        match value {
            Ok(val) => Ok(val),
            Err(e) => Err((*e).clone()),
        }
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

#[tracing::instrument]
fn deserialize_from_value<T>(value: serde_json::Value) -> Result<T, Error>
where
    T: serde::de::DeserializeOwned + std::fmt::Debug,
{
    match serde_json::from_value::<T>(value).map_err(From::from) {
        Ok(item) => {
            tracing::debug!(object = ?item, "Received valid object");
            Ok(item)
        }
        e => {
            tracing::warn!(error= ?e, "Object failed deserialization");
            e
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use freedom_config::FreedomConfig;
    use futures::StreamExt;
    use tracing_test::traced_test;

    fn dev_client() -> CachingClient {
        let cache = moka::future::CacheBuilder::new(10).build();

        let config = FreedomConfig::from_env().unwrap();
        CachingClient {
            inner: Client::new(config),
            cache,
        }
    }

    #[tokio::test]
    #[traced_test]
    async fn task_by_id_speed_test() {
        let mut client = dev_client();
        // First query hits Freedom API
        let now = std::time::Instant::now();
        assert!(client.get_task_by_id(17812).await.is_ok());
        let non_cache_duration = now.elapsed();
        tracing::debug!("Non cache duration: {:?}", non_cache_duration);

        // Second query hits cache
        let now = std::time::Instant::now();
        assert!(client.get_task_by_id(17812).await.is_ok());
        let cache_duration = now.elapsed();
        tracing::debug!("Cache duration: {:?}", cache_duration);

        assert!(cache_duration < non_cache_duration);
    }

    #[tokio::test]
    #[traced_test]
    async fn paginated_users_speed_test() {
        let mut client = dev_client();
        // First query hits Freedom API
        let now = std::time::Instant::now();
        let stream_boi = client.get_users().collect::<Vec<_>>().await;
        for i in stream_boi {
            i.unwrap();
        }
        let non_cache_duration = now.elapsed();
        tracing::debug!("Non cache duration: {:?}", non_cache_duration);

        // Second query hits cache
        let now = std::time::Instant::now();
        let stream_boi = client.get_users().collect::<Vec<_>>().await;
        for i in stream_boi {
            i.unwrap();
        }
        let cache_duration = now.elapsed();
        tracing::debug!("Cache duration: {:?}", cache_duration);

        assert!(cache_duration < non_cache_duration);
    }
}
