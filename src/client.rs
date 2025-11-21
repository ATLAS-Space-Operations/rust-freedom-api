use bytes::Bytes;
use freedom_config::Config;
use reqwest::{RequestBuilder, Response, StatusCode};
use url::Url;

use crate::api::{Api, Inner, Value};

/// An asynchronous `Client` for interfacing with the ATLAS freedom API.
///
/// The client is primarily defined based on it's [`Env`](crate::config::Env)
/// and it's credentials (username and password).
#[derive(Clone, Debug)]
pub struct Client {
    pub(crate) config: Config,
    pub(crate) client: reqwest::Client,
    universal_headers: Vec<(String, String)>,
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
            universal_headers: Vec::new(),
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

    /// Adds a universal header key and value to all GET POST, and DELETEs made with the client
    pub fn with_universal_header(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.universal_headers.push((key.into(), value.into()));
        self
    }

    fn append_headers(&self, mut req: RequestBuilder) -> RequestBuilder {
        for (header, value) in self.universal_headers.iter() {
            req = req.header(header, value);
        }
        req
    }
}

impl Api for Client {
    type Container<T: Value> = Inner<T>;

    async fn get(&self, url: Url) -> Result<(Bytes, StatusCode), crate::error::Error> {
        tracing::trace!("GET to {}", url);

        let req = self.append_headers(self.client.get(url.clone()));

        let resp = req
            .basic_auth(self.config.key(), Some(&self.config.expose_secret()))
            .send()
            .await?;

        let status = resp.status();
        let body = resp
            .bytes()
            .await
            .inspect_err(|error| tracing::warn!(%url, %error, %status, "Failed to get response body"))
            .inspect(|body| tracing::info!(%url, body = %String::from_utf8_lossy(body), %status, "Received response body"))?;

        Ok((body, status))
    }

    async fn delete(&self, url: Url) -> Result<Response, crate::error::Error> {
        tracing::trace!("DELETE to {}", url);

        let req = self.append_headers(self.client.delete(url.clone()));

        req.basic_auth(self.config.key(), Some(self.config.expose_secret()))
            .send()
            .await
            .inspect_err(|error| tracing::warn!(%error, %url, "Failed to DELETE"))
            .inspect(|ok| tracing::warn!(?ok, %url, "Received response"))
            .map_err(From::from)
    }

    async fn post<S>(&self, url: Url, msg: S) -> Result<Response, crate::error::Error>
    where
        S: serde::Serialize + Sync + Send,
    {
        tracing::trace!("POST to {}", url);

        let req = self.append_headers(self.client.post(url.clone()));

        req.basic_auth(self.config.key(), Some(self.config.expose_secret()))
            .json(&msg)
            .send()
            .await
            .inspect_err(|error| tracing::warn!(%error, %url, "Failed to POST"))
            .inspect(|ok| tracing::warn!(?ok, %url, "Received response"))
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

    use crate::Container;

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
        let inner = Inner::new(val.clone());
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
        mock.assert_calls(1);
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
        mock.assert_calls(1);
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

        mock.assert_calls(1);
    }
}
