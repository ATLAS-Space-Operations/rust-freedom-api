use bytes::Bytes;
use freedom_config::Config;
use reqwest::{Response, StatusCode};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::api::{Api, Container, Value};

/// An asynchronous `Client` for interfacing with the ATLAS freedom API.
///
/// The client is primarily defined based on it's [`Env`](crate::config::Env)
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
    /// Construct an API client from the provided Freedom config
    ///
    /// # Example
    ///
    /// ```
    /// # use freedom_api::prelude::*;
    /// let config = Config::builder()
    ///     .environment(Test)
    ///     .key("foo")
    ///     .secret("bar")
    ///     .build()
    ///     .unwrap();
    /// let client = Client::from_config(config);
    ///
    /// assert_eq!(client.config().key(), "foo");
    /// ```
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

        if !status.is_success() {
            let readable_body = String::from_utf8_lossy(&body);
            return Err(crate::error::Error::Response(format!(
                "{}\n{readable_body}",
                status
            )));
        }

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

#[cfg(test)]
mod tests {
    use freedom_config::Test;
    use httpmock::{
        Method::{GET, POST},
        MockServer,
    };

    use super::*;

    fn default_client() -> Client {
        let config = Config::builder()
            .environment(Test)
            .key("foo")
            .secret("bar")
            .build()
            .unwrap();

        Client::from_config(config)
    }

    #[test]
    fn clients_are_eq_based_on_config() {
        let config = Config::builder()
            .environment(Test)
            .key("foo")
            .secret("bar")
            .build()
            .unwrap();

        let client_1 = Client::from_config(config.clone());
        let client_2 = Client::from_config(config);
        assert_eq!(client_1, client_2);
    }

    #[test]
    fn wrap_and_unwrap_inner() {
        let val = String::from("foobar");
        let inner = Inner(val.clone());
        assert_eq!(*inner, val);
        let unwrapped = inner.into_inner();
        assert_eq!(val, unwrapped);
    }

    #[test]
    fn load_from_env() {
        unsafe {
            std::env::set_var("ATLAS_ENV", "TEST");
            std::env::set_var("ATLAS_KEY", "foo");
            std::env::set_var("ATLAS_SECRET", "bar");
        };
        let client = Client::from_env().unwrap();
        assert_eq!(client.config().key(), "foo");
        assert_eq!(client.config().expose_secret(), "bar");
    }

    #[tokio::test]
    async fn get_ok_response() {
        const RESPONSE: &str = "it's working";
        let client = default_client();
        let server = MockServer::start();
        let addr = server.address();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/testing");
            then.body(RESPONSE.as_bytes());
        });
        let url = Url::parse(&format!("http://{}/testing", addr)).unwrap();
        let (response, status) = client.get(url).await.unwrap();

        assert_eq!(response, RESPONSE.as_bytes());
        assert_eq!(status, StatusCode::OK);
        mock.assert_hits(1);
    }

    #[tokio::test]
    async fn get_err_response() {
        const RESPONSE: &str = "NOPE";
        let client = default_client();
        let server = MockServer::start();
        let addr = server.address();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/testing");
            then.body(RESPONSE.as_bytes()).status(404);
        });
        let url = Url::parse(&format!("http://{}/testing", addr)).unwrap();
        let (response, status) = client.get(url).await.unwrap();

        assert_eq!(response, RESPONSE.as_bytes());
        assert_eq!(status, StatusCode::NOT_FOUND);
        mock.assert_hits(1);
    }

    #[tokio::test]
    async fn post_json() {
        let client = default_client();
        let server = MockServer::start();
        let addr = server.address();
        let json = serde_json::json!({
            "name": "foo",
            "data": 12
        });
        let json_clone = json.clone();
        let mock = server.mock(|when, then| {
            when.method(POST).path("/testing").json_body(json_clone);
            then.body(b"OK").status(200);
        });
        let url = Url::parse(&format!("http://{}/testing", addr)).unwrap();
        client.post(url, &json).await.unwrap();

        mock.assert_hits(1);
    }
}
