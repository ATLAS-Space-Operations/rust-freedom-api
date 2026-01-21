//! # ATLAS Freedom API
//!
//! This module exists to define the Freedom API trait, which can be implemented for multiple client
//! types.
//!
//! The API trait
#![allow(clippy::type_complexity)]
use std::{future::Future, ops::Deref, pin::Pin};

use async_stream::stream;
use bytes::Bytes;
use freedom_config::Config;
use freedom_models::{
    account::Account,
    band::Band,
    pagination::Paginated,
    satellite::Satellite,
    satellite_configuration::SatelliteConfiguration,
    site::{Site, SiteConfiguration},
    task::{Task, TaskRequest, TaskStatusType, TaskType},
    user::User,
    utils::Embedded,
};
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde_json::Value as JsonValue;
use time::{OffsetDateTime, format_description::well_known::Iso8601};
use url::Url;

use futures_core::Stream;

use crate::error::Error;

pub(crate) mod post;

/// A super trait containing all the requirements for Freedom API Values
pub trait Value: std::fmt::Debug + DeserializeOwned + Clone + Send + Sync {}

impl<T> Value for T where T: std::fmt::Debug + DeserializeOwned + Clone + Send + Sync {}

trait PaginatedErr<'a, T> {
    fn once_err(self) -> PaginatedStream<'a, T>;
}

impl<'a, T: 'a + Send + Sync> PaginatedErr<'a, T> for Error {
    fn once_err(self) -> PaginatedStream<'a, T> {
        Box::pin(async_stream::stream! { yield Err(self); })
    }
}

/// The trait defining the required functionality of container types
///
/// The Freedom API is generic over "containers". Each implementer of the [`Api`] trait must
/// also define a container. This is useful since certain clients will return Arc'd values, i.e. the
/// caching client, while others return the values wrapped in a simple [`Inner`] type which is just
/// a stack value.
///
/// However, for most cases this complexity can be ignored, since containers are required to
/// implement [`Deref`](std::ops::Deref) of `T`. So for read-only operations the container can be
/// used as if it were `T`. For mutable access see [`Self::into_inner`].
///
/// # Example
///
/// ```no_run
/// # use freedom_api::prelude::*;
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let config = Config::from_env()?;
/// # let client = Client::from_config(config);
/// let request = client
///     .get_request_by_id(42)
///     .await?;
///
/// println!("Created on {}", request.created); // Direct access to created field
///                                             // through the Container
/// # Ok(())
/// # }
/// ```
pub trait Container<T>: Deref<Target = T> + Value {
    /// All containers are capable of returning the value they wrap
    ///
    /// However, the runtime performance of this varies by client type. For [`crate::Client`], this
    /// operation is essentially free, however for the caching client, this often results in a clone
    /// of the value.
    fn into_inner(self) -> T;
}

impl<T: Deref<Target = T> + Value> Container<T> for Box<T> {
    fn into_inner(self) -> T {
        *self
    }
}

/// A simple container which stores a `T`.
///
/// This container exists to allow us to store items on the stack, without needing to allocate with
/// something like `Box<T>`. For all other intents and purposes, it acts as the `T` which it
/// contains.
#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
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

impl<T> Inner<T> {
    pub fn new(inner: T) -> Self {
        Self(inner)
    }
}

/// A stream of paginated results from freedom.
///
/// Each item in the stream is a result, since one or more items may fail to be serialized
pub type PaginatedStream<'a, T> = Pin<Box<dyn Stream<Item = Result<T, Error>> + 'a + Send + Sync>>;

/// The primary trait for interfacing with the Freedom API
pub trait Api: Send + Sync {
    /// The [`Api`] supports implementors with different so-called "container" types.
    ///
    /// For a more detailed description, see the [`Container`] trait.
    type Container<T: Value>: Container<T>;

    /// Returns the freedom configuration for the API
    fn config(&self) -> &Config;

    /// Returns a mutable reference to the freedom configuration for the API
    fn config_mut(&mut self) -> &mut Config;

    /// Creates a get request at the provided absolute URI for the client's environment, using basic
    /// authentication.
    ///
    /// Returns the raw binary body, and the status code.
    fn get(
        &self,
        url: Url,
    ) -> impl Future<Output = Result<(Bytes, StatusCode), Error>> + Send + Sync;

    fn delete(&self, url: Url) -> impl Future<Output = Result<(Bytes, StatusCode), Error>> + Send;

    /// Lower level method, not intended for direct use
    fn post<S>(
        &self,
        url: Url,
        msg: S,
    ) -> impl Future<Output = Result<(Bytes, StatusCode), Error>> + Send + Sync
    where
        S: serde::Serialize + Send + Sync;

    /// Creates a get request at the provided absolute URI for the client's environment, using basic
    /// authentication.
    ///
    /// The JSON response is then deserialized into the required type, erroring if the
    /// deserialization fails, and providing the object if it succeeds.
    fn get_json_map<T>(&self, url: Url) -> impl Future<Output = Result<T, Error>> + Send + Sync
    where
        T: Value,
    {
        async move {
            let (body, status) = self.get(url).await?;

            error_on_non_success(&status, &body)?;

            let utf8_str = String::from_utf8_lossy(&body);
            serde_json::from_str(&utf8_str).map_err(From::from)
        }
    }

    /// Creates a stream of items from a paginated endpoint.
    ///
    /// The stream is produced as a collection of `Result<T>`. This is so that if any one item fails
    /// deserialization, it is added to the stream of items as an error rather than causing the
    /// entire stream to result in an Error.
    ///
    /// # Pinning
    ///
    /// For convenience the stream is pinned on the heap via [`Box::pin`](https://doc.rust-lang.org/std/boxed/struct.Box.html#method.pin).
    /// This allows us to treat the returned stream more like any other object, without requiring
    /// the end user to manually  pin the result on the stack. This comes with a slight performance
    /// penalty (it requires an allocation), however this will be negligible given the latency of
    /// the responses. For more information on pinning in rust refer to the [pinning chapter](https://rust-lang.github.io/async-book/04_pinning/01_chapter.html)
    /// of the async book.
    fn get_paginated<T>(&self, head_url: Url) -> PaginatedStream<'_, Self::Container<T>>
    where
        T: 'static + Value + Send + Sync,
    {
        let base = self.config().environment().freedom_entrypoint();
        let mut current_url = head_url; // Not necessary but makes control flow more obvious
        Box::pin(stream! {
            loop {
                // Get the results for the current page.
                let pag = self.get_json_map::<Paginated<JsonValue>>(current_url).await?;
                for item in pag.items {
                    let i = serde_json::from_value::<Self::Container<T>>(item).map_err(From::from);
                    yield i;
                }
                if let Some(link) = pag.links.get("next") {
                    // Update the URL to the next page.
                    current_url = match link.has_host() {
                        true => link.to_owned(),
                        false => {
                            base.clone()
                                .join(link.as_str())
                                .map_err(|e| crate::error::Error::pag_item(e.to_string()))?
                        }
                    };
                } else {
                    break;
                }
            }
        })
    }

    /// Fetch the URL from the given path
    fn path_to_url(&self, path: impl AsRef<str>) -> Result<Url, Error> {
        let url = self.config().environment().freedom_entrypoint();
        url.join(path.as_ref())
            .map_err(|error| Error::InvalidUri(error.to_string()))
    }

    /// Request to delete the band details object matching the provided id
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use freedom_api::prelude::*;
    /// # tokio_test::block_on(async {
    /// let client = Client::from_env()?;
    ///
    /// client.delete_task_request(42).await?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// # });
    /// ```
    fn delete_band_details(&self, id: i32) -> impl Future<Output = Result<(), Error>> + Send {
        async move {
            let uri = self.path_to_url(format!("satellite_bands/{id}"))?;
            let (body, status) = self.delete(uri).await?;
            error_on_non_success(&status, &body)?;
            Ok(())
        }
    }

    /// Request to delete the satellite configuration matching the provided `id`
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use freedom_api::prelude::*;
    /// # tokio_test::block_on(async {
    /// let client = Client::from_env()?;
    ///
    /// client.delete_satellite_configuration(42).await?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// # });
    /// ```
    fn delete_satellite_configuration(
        &self,
        id: i32,
    ) -> impl Future<Output = Result<(), Error>> + Send {
        async move {
            let uri = self.path_to_url(format!("satellite_configurations/{id}"))?;
            let (body, status) = self.delete(uri).await?;
            error_on_non_success(&status, &body)?;
            Ok(())
        }
    }

    /// Request to delete the satellite object matching the provided `id`
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use freedom_api::prelude::*;
    /// # tokio_test::block_on(async {
    /// let client = Client::from_env()?;
    ///
    /// client.delete_satellite(42).await?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// # });
    /// ```
    fn delete_satellite(&self, id: i32) -> impl Future<Output = Result<(), Error>> + Send {
        async move {
            let uri = self.path_to_url(format!("satellites/{id}"))?;
            let (body, status) = self.delete(uri).await?;
            error_on_non_success(&status, &body)?;
            Ok(())
        }
    }

    /// Request to delete the override matching the provided `id`
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use freedom_api::prelude::*;
    /// # tokio_test::block_on(async {
    /// let client = Client::from_env()?;
    ///
    /// client.delete_override(42).await?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// # });
    /// ```
    fn delete_override(&self, id: i32) -> impl Future<Output = Result<(), Error>> + Send {
        async move {
            let uri = self.path_to_url(format!("overrides/{id}"))?;
            let (body, status) = self.delete(uri).await?;
            error_on_non_success(&status, &body)?;
            Ok(())
        }
    }

    /// Request to delete the user matching the provided `id`
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use freedom_api::prelude::*;
    /// # tokio_test::block_on(async {
    /// let client = Client::from_env()?;
    ///
    /// client.delete_user(42).await?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// # });
    /// ```
    fn delete_user(&self, id: i32) -> impl Future<Output = Result<(), Error>> + Send {
        async move {
            let uri = self.path_to_url(format!("users/{id}"))?;
            let (body, status) = self.delete(uri).await?;
            error_on_non_success(&status, &body)?;
            Ok(())
        }
    }

    /// Request to delete the user matching the provided `id`
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use freedom_api::prelude::*;
    /// # tokio_test::block_on(async {
    /// let client = Client::from_env()?;
    ///
    /// client.delete_task_request(42).await?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// # });
    /// ```
    fn delete_task_request(&self, id: i32) -> impl Future<Output = Result<(), Error>> + Send {
        async move {
            let uri = self.path_to_url(format!("requests/{id}"))?;
            let (body, status) = self.delete(uri).await?;
            error_on_non_success(&status, &body)?;
            Ok(())
        }
    }

    /// Lower level method, not intended for direct use
    fn post_json_map<S, T>(
        &self,
        url: Url,
        msg: S,
    ) -> impl Future<Output = Result<T, Error>> + Send + Sync
    where
        S: serde::Serialize + Send + Sync,
        T: Value,
    {
        async move {
            let (body, status) = self.post(url, msg).await?;

            error_on_non_success(&status, &body)?;

            let utf8_str = String::from_utf8_lossy(&body);
            serde_json::from_str(&utf8_str).map_err(From::from)
        }
    }

    /// Produces a single [`Account`](freedom_models::account::Account) matching the provided ID.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use freedom_api::prelude::*;
    /// # tokio_test::block_on(async {
    /// let client = Client::from_env()?;
    ///
    /// let account = client.get_account_by_name("ATLAS").await?;
    /// println!("{}", account.name);
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// # });
    /// ```
    fn get_account_by_name(
        &self,
        account_name: &str,
    ) -> impl Future<Output = Result<Self::Container<Account>, Error>> + Send + Sync {
        async move {
            let mut uri = self.path_to_url("accounts/search/findOneByName")?;
            uri.set_query(Some(&format!("name={account_name}")));
            self.get_json_map(uri).await
        }
    }

    /// Produces a single [`Account`](freedom_models::account::Account) matching the provided ID.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use freedom_api::prelude::*;
    /// # tokio_test::block_on(async {
    /// let client = Client::from_env()?;
    ///
    /// let data = client.get_file_by_task_id_and_name(42, "data.bin").await?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// # });
    /// ```
    fn get_file_by_task_id_and_name(
        &self,
        task_id: i32,
        file_name: &str,
    ) -> impl Future<Output = Result<Bytes, Error>> + Send + Sync {
        async move {
            let path = format!("downloads/{}/{}", task_id, file_name);
            let uri = self.path_to_url(path)?;

            let (data, status) = self.get(uri).await?;
            error_on_non_success(&status, b"Failed to fetch file")?;

            Ok(data)
        }
    }

    /// Produces a single [`Account`](freedom_models::account::Account) matching the provided ID.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    fn get_account_by_id(
        &self,
        account_id: i32,
    ) -> impl Future<Output = Result<Self::Container<Account>, Error>> + Send + Sync {
        async move {
            let uri = self.path_to_url(format!("accounts/{account_id}"))?;
            self.get_json_map(uri).await
        }
    }

    /// Produces a paginated stream of [`Account`](freedom_models::account::Account) objects.
    ///
    /// See [`get_paginated`](Self::get_paginated) documentation for more details about the process
    /// and return type
    fn get_accounts(&self) -> PaginatedStream<'_, Self::Container<Account>> {
        let uri = match self.path_to_url("accounts") {
            Ok(uri) => uri,
            Err(err) => return err.once_err(),
        };
        self.get_paginated(uri)
    }

    /// Produces a paginated stream of [`Band`] objects.
    ///
    /// See [`get_paginated`](Self::get_paginated) documentation for more details about the process
    /// and return type
    fn get_satellite_bands(&self) -> PaginatedStream<'_, Self::Container<Band>> {
        let uri = match self.path_to_url("satellite_bands") {
            Ok(uri) => uri,
            Err(err) => return err.once_err(),
        };
        self.get_paginated(uri)
    }

    /// Produces a single [`Band`] matching the provided ID.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    fn get_satellite_band_by_id(
        &self,
        satellite_band_id: i32,
    ) -> impl Future<Output = Result<Self::Container<Band>, Error>> + Send + Sync {
        async move {
            let uri = self.path_to_url(format!("satellite_bands/{satellite_band_id}"))?;
            self.get_json_map(uri).await
        }
    }

    /// Produces a single [`Band`] matching the provided name.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    fn get_satellite_band_by_name(
        &self,
        satellite_band_name: &str,
    ) -> impl Future<Output = Result<Self::Container<Band>, Error>> + Send + Sync {
        async move {
            let mut uri = self.path_to_url("satellite_bands/search/findOneByName")?;
            uri.set_query(Some(&format!("name={satellite_band_name}")));
            self.get_json_map(uri).await
        }
    }

    /// Produces a paginated stream of [`Band`] objects matching the provided account name.
    ///
    /// See [`get_paginated`](Self::get_paginated) documentation for more details about the process
    /// and return type
    fn get_satellite_bands_by_account_name(
        &self,
        account_name: &str,
    ) -> PaginatedStream<'_, Self::Container<Band>> {
        let mut uri = match self.path_to_url("satellite_bands/search/findAllByAccountName") {
            Ok(uri) => uri,
            Err(err) => return err.once_err(),
        };
        uri.set_query(Some(&format!("accountName={account_name}")));

        self.get_paginated(uri)
    }

    /// Produces a paginated stream of [`SatelliteConfiguration`] objects matching the provided
    /// account name.
    ///
    /// See [`get_paginated`](Self::get_paginated) documentation for more details about the process
    /// and return type
    fn get_satellite_configurations_by_account_name(
        &self,
        account_name: &str,
    ) -> PaginatedStream<'_, Self::Container<SatelliteConfiguration>> {
        let mut uri = match self.path_to_url("satellite_configurations/search/findAllByAccountName")
        {
            Ok(uri) => uri,
            Err(err) => return err.once_err(),
        };
        uri.set_query(Some(&format!("accountName={account_name}")));

        self.get_paginated(uri)
    }

    /// Produces a paginated stream of [`SatelliteConfiguration`] objects.
    ///
    /// See [`get_paginated`](Self::get_paginated) documentation for more details about the process
    /// and return type
    fn get_satellite_configurations(
        &self,
    ) -> PaginatedStream<'_, Self::Container<SatelliteConfiguration>> {
        let uri = match self.path_to_url("satellite_configurations") {
            Ok(uri) => uri,
            Err(err) => return err.once_err(),
        };

        self.get_paginated(uri)
    }

    /// Produces a single satellite configuration matching the provided satellite configuration ID
    fn get_satellite_configuration_by_id(
        &self,
        satellite_configuration_id: i32,
    ) -> impl Future<Output = Result<Self::Container<SatelliteConfiguration>, Error>> + Send + Sync
    {
        async move {
            let uri = self.path_to_url(format!(
                "satellite_configurations/{satellite_configuration_id}"
            ))?;

            self.get_json_map(uri).await
        }
    }

    /// Produces a single satellite configuration matching the provided satellite configuration name
    fn get_satellite_configuration_by_name(
        &self,
        satellite_configuration_name: &str,
    ) -> impl Future<Output = Result<Self::Container<SatelliteConfiguration>, Error>> + Send + Sync
    {
        async move {
            let mut uri = self.path_to_url("satellite_configurations/search/findOneByName")?;
            uri.set_query(Some(&format!("name={satellite_configuration_name}")));

            self.get_json_map(uri).await
        }
    }

    /// Produces a paginated stream of [`Site`] objects.
    ///
    /// See [`get_paginated`](Self::get_paginated) documentation for more details about the process
    /// and return type
    fn get_sites(&self) -> PaginatedStream<'_, Self::Container<Site>> {
        let uri = match self.path_to_url("sites") {
            Ok(uri) => uri,
            Err(err) => return err.once_err(),
        };
        self.get_paginated(uri)
    }

    /// Produces a single [`Site`] object matching the provided ID.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    fn get_site_by_id(
        &self,
        id: i32,
    ) -> impl Future<Output = Result<Self::Container<Site>, Error>> + Send + Sync {
        async move {
            let uri = self.path_to_url(format!("sites/{id}"))?;
            self.get_json_map(uri).await
        }
    }

    /// Produces a single [`Site`] object matching the provided name.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    fn get_site_by_name(
        &self,
        name: impl AsRef<str> + Send + Sync,
    ) -> impl Future<Output = Result<Self::Container<Site>, Error>> + Send + Sync {
        async move {
            let mut uri = self.path_to_url("sites/search/findOneByName")?;
            let query = format!("name={}", name.as_ref());
            uri.set_query(Some(&query));

            self.get_json_map(uri).await
        }
    }

    /// Produces a paginated stream of [`SiteConfiguration`] objects.
    ///
    /// See [`get_paginated`](Self::get_paginated) documentation for more details about the process
    /// and return type
    fn get_site_configurations(&self) -> PaginatedStream<'_, Self::Container<SiteConfiguration>> {
        let uri = match self.path_to_url("configurations") {
            Ok(uri) => uri,
            Err(err) => return err.once_err(),
        };
        self.get_paginated(uri)
    }

    /// Produces a single [`SiteConfiguration`] object matching the provided ID.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    fn get_site_configuration_by_id(
        &self,
        id: i32,
    ) -> impl Future<Output = Result<Self::Container<SiteConfiguration>, Error>> + Send + Sync {
        async move {
            let uri = self.path_to_url(format!("configurations/{id}"))?;
            self.get_json_map(uri).await
        }
    }

    /// Produces a single [`SiteConfiguration`] object matching the provided name.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    fn get_site_configuration_by_name(
        &self,
        name: impl AsRef<str> + Send + Sync,
    ) -> impl Future<Output = Result<Self::Container<SiteConfiguration>, Error>> + Send + Sync {
        async move {
            let mut uri = self.path_to_url("configurations/search/findOneByName")?;
            let query = format!("name={}", name.as_ref());
            uri.set_query(Some(&query));

            self.get_json_map(uri).await
        }
    }

    /// Produces a single [`TaskRequest`] matching the provided ID.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    fn get_request_by_id(
        &self,
        task_request_id: i32,
    ) -> impl Future<Output = Result<Self::Container<TaskRequest>, Error>> + Send + Sync {
        async move {
            let uri = self.path_to_url(format!("requests/{task_request_id}"))?;

            self.get_json_map(uri).await
        }
    }

    /// Produces a paginated stream of [`TaskRequest`] objects.
    ///
    /// See [`get_paginated`](Self::get_paginated) documentation for more details about the process
    /// and return type
    fn get_requests(&self) -> PaginatedStream<'_, Self::Container<TaskRequest>> {
        {
            let uri = match self.path_to_url("requests/search/findAll") {
                Ok(uri) => uri,
                Err(err) => return err.once_err(),
            };
            self.get_paginated(uri)
        }
    }

    /// Produces a vector of [`TaskRequest`] items, representing all the task requests matching the
    /// target time overlapping with the provided time range.
    fn get_requests_by_target_date_between(
        &self,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> impl Future<Output = Result<Self::Container<Vec<TaskRequest>>, Error>> + Send + Sync {
        async move {
            let mut uri = self.path_to_url("requests/search/findAllByTargetDateBetween")?;

            uri.set_query(Some(&format!(
                "start={}&end={}",
                start.format(&Iso8601::DEFAULT)?,
                end.format(&Iso8601::DEFAULT)?,
            )));

            self.get_json_map(uri).await
        }
    }

    /// Produces a vector of [`TaskRequest`] items,
    /// representing all the task requests matching the account at the provided URI and whose
    /// target time overlaps with the provided time range.
    ///
    /// See [`get_paginated`](Self::get_paginated) documentation for more details about the process
    /// and return type
    fn get_requests_by_account_and_target_date_between<T>(
        &self,
        account_uri: T,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> PaginatedStream<'_, Self::Container<TaskRequest>>
    where
        T: AsRef<str> + Send + Sync,
    {
        let start = match start.format(&Iso8601::DEFAULT).map_err(Error::from) {
            Ok(start) => start,
            Err(error) => return error.once_err(),
        };

        let end = match end.format(&Iso8601::DEFAULT).map_err(Error::from) {
            Ok(end) => end,
            Err(error) => return error.once_err(),
        };

        let mut uri = match self.path_to_url("requests/search/findAllByAccountAndTargetDateBetween")
        {
            Ok(uri) => uri,
            Err(err) => return err.once_err(),
        };

        uri.set_query(Some(&format!(
            "account={}&start={}&end={}",
            account_uri.as_ref(),
            start,
            end
        )));

        self.get_paginated(uri)
    }

    /// Produces a paginated stream of [`TaskRequest`]
    /// objects whose account name matches the provided name, and whose pass will occur today.
    ///
    /// See [`get_paginated`](Self::get_paginated) documentation for more details about the process
    /// and return type
    fn get_requests_by_account_and_upcoming_today(
        &self,
    ) -> PaginatedStream<'_, Self::Container<TaskRequest>> {
        let uri = match self.path_to_url("requests/search/findByAccountUpcomingToday") {
            Ok(uri) => uri,
            Err(err) => return err.once_err(),
        };

        self.get_paginated(uri)
    }

    /// Produces a paginated stream of [`TaskRequest`]
    /// objects whose satellite configuration matches that of the configuration at the
    /// `configuration_uri` endpoint.
    ///
    /// See [`get_paginated`](Self::get_paginated) documentation for more details about the process
    /// and return type
    ///
    /// # Note
    /// The results are ordered by the creation time of the task request
    fn get_requests_by_configuration<T>(
        &self,
        configuration_uri: T,
    ) -> PaginatedStream<'_, Self::Container<TaskRequest>>
    where
        T: AsRef<str> + Send + Sync,
    {
        let mut uri =
            match self.path_to_url("requests/search/findAllByConfigurationOrderByCreatedAsc") {
                Ok(uri) => uri,
                Err(err) => return err.once_err(),
            };

        uri.set_query(Some(&format!(
            "configuration={}",
            configuration_uri.as_ref()
        )));

        self.get_paginated::<TaskRequest>(uri)
    }

    /// Produces a vector of [`TaskRequest`] items, representing all the task requests which match
    /// the provided configuration, whose satellite name matches one of the names provided as part
    /// of `satellite_name`, and which overlaps the provided time range.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    fn get_requests_by_configuration_and_satellite_names_and_target_date_between<T, I, S>(
        &self,
        configuration_uri: T,
        satellites: I,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> impl Future<Output = Result<Self::Container<Vec<TaskRequest>>, Error>> + Send + Sync
    where
        T: AsRef<str> + Send + Sync,
        I: IntoIterator<Item = S> + Send + Sync,
        S: AsRef<str> + Send + Sync,
    {
        async move {
            let satellites_string = crate::utils::list_to_string(satellites);
            let mut uri = self.path_to_url(
                "requests/search/findAllByConfigurationAndSatelliteNamesAndTargetDateBetween",
            )?;

            uri.set_query(Some(&format!(
                "configuration={}&satelliteNames={}&start={}&end={}",
                configuration_uri.as_ref(),
                satellites_string,
                start.format(&Iso8601::DEFAULT)?,
                end.format(&Iso8601::DEFAULT)?,
            )));

            Ok(self
                .get_json_map::<Embedded<Self::Container<Vec<TaskRequest>>>>(uri)
                .await?
                .items)
        }
    }

    /// Produces a vector of [`TaskRequest`] items, representing all the task requests matching the
    /// configuration at the provided URI and whose target time overlaps with the provided time
    /// range.
    ///
    /// See [`get_paginated`](Self::get_paginated) documentation for more details about the process
    /// and return type
    fn get_requests_by_configuration_and_target_date_between<T>(
        &self,
        configuration_uri: T,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> impl Future<Output = Result<Self::Container<Vec<TaskRequest>>, Error>> + Send + Sync
    where
        T: AsRef<str> + Send + Sync,
    {
        async move {
            let mut uri =
                self.path_to_url("requests/search/findAllByConfigurationAndTargetDateBetween")?;
            uri.set_query(Some(&format!(
                "configuration={}&start={}&end={}",
                configuration_uri.as_ref(),
                start.format(&Iso8601::DEFAULT)?,
                end.format(&Iso8601::DEFAULT)?,
            )));

            Ok(self
                .get_json_map::<Embedded<Self::Container<Vec<TaskRequest>>>>(uri)
                .await?
                .items)
        }
    }

    /// Produces a vector of [`TaskRequest`] items,
    /// representing all the task requests whose ID matches one of the IDs provided as part of
    /// `ids`.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    fn get_requests_by_ids<I, S>(
        &self,
        ids: I,
    ) -> impl Future<Output = Result<Self::Container<Vec<TaskRequest>>, Error>> + Send + Sync
    where
        I: IntoIterator<Item = S> + Send + Sync,
        S: AsRef<str> + Send + Sync,
    {
        async move {
            let ids_string = crate::utils::list_to_string(ids);
            let mut uri = self.path_to_url("requests/search/findAllByIds")?;

            uri.set_query(Some(&format!("ids={}", ids_string)));

            Ok(self
                .get_json_map::<Embedded<Self::Container<Vec<TaskRequest>>>>(uri)
                .await?
                .items)
        }
    }

    /// Produces a paginated stream of [`TaskRequest`] objects which are public, and which overlap
    /// with the provided time range.
    ///
    /// See [`get_paginated`](Self::get_paginated) documentation for more details about the process
    /// and return type
    fn get_requests_by_overlapping_public(
        &self,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> PaginatedStream<'_, Self::Container<TaskRequest>> {
        let mut uri = match self.path_to_url("requests/search/findAllByOverlappingPublic") {
            Ok(uri) => uri,
            Err(err) => return err.once_err(),
        };

        uri.set_query(Some(&format!(
            "start={}&end={}",
            start.format(&Iso8601::DEFAULT).unwrap(),
            end.format(&Iso8601::DEFAULT).unwrap(),
        )));

        self.get_paginated(uri)
    }

    /// Produces a paginated stream of [`TaskRequest`] objects whose satellite name matches one of
    /// the names provided as part of `satellite_name`.
    ///
    /// See [`get_paginated`](Self::get_paginated) documentation for more details about the process
    /// and return type
    fn get_requests_by_satellite_name<T>(
        &self,
        satellite_name: T,
    ) -> PaginatedStream<'_, Self::Container<TaskRequest>>
    where
        T: AsRef<str> + Send + Sync,
    {
        let mut uri = match self.path_to_url("requests/search/findBySatelliteName") {
            Ok(uri) => uri,
            Err(err) => return err.once_err(),
        };

        uri.set_query(Some(&format!("name={}", satellite_name.as_ref())));

        self.get_paginated(uri)
    }

    /// Produces a vector of [`TaskRequest`] items, representing all the task requests whose
    /// satellite name matches the provided name and whose target time overlaps with the provided
    /// time range.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    fn get_requests_by_satellite_name_and_target_date_between<T>(
        &self,
        satellite_name: T,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> impl Future<Output = Result<Self::Container<Vec<TaskRequest>>, Error>> + Send + Sync
    where
        T: AsRef<str> + Send + Sync,
    {
        async move {
            let mut uri =
                self.path_to_url("requests/search/findAllBySatelliteNameAndTargetDateBetween")?;

            uri.set_query(Some(&format!(
                "name={}&start={}&end={}",
                satellite_name.as_ref(),
                start.format(&Iso8601::DEFAULT)?,
                end.format(&Iso8601::DEFAULT)?
            )));

            Ok(self
                .get_json_map::<Embedded<Self::Container<Vec<TaskRequest>>>>(uri)
                .await?
                .items)
        }
    }

    /// Produces a paginated stream of [`TaskRequest`] objects whose status matches the provided
    /// status.
    ///
    /// See [`get_paginated`](Self::get_paginated) documentation for more details about the process
    /// and return type
    fn get_requests_by_status<T>(
        &self,
        status: T,
    ) -> PaginatedStream<'_, Self::Container<TaskRequest>>
    where
        T: TryInto<TaskStatusType> + Send + Sync,
        Error: From<<T as TryInto<TaskStatusType>>::Error>,
    {
        let status: TaskStatusType = match status.try_into() {
            Ok(val) => val,
            Err(err) => return Error::from(err).once_err(),
        };
        let mut uri = match self.path_to_url("requests/search/findByStatus") {
            Ok(uri) => uri,
            Err(err) => return err.once_err(),
        };

        uri.set_query(Some(&format!("status={}", status.as_ref())));

        self.get_paginated(uri)
    }

    /// Produces a paginated stream of [`TaskRequest`], representing all the task requests which
    /// match the provided status, account, and overlap the provided time range.
    ///
    /// See [`get_paginated`](Self::get_paginated) documentation for more details about the process
    /// and return type
    fn get_requests_by_status_and_account_and_target_date_between<T, U>(
        &self,
        status: T,
        account_uri: U,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> PaginatedStream<'_, Self::Container<TaskRequest>>
    where
        T: AsRef<str> + Send + Sync,
        U: AsRef<str> + Send + Sync,
    {
        let mut uri = match self
            .path_to_url("requests/search/findAllByStatusAndAccountAndTargetDateBetween")
        {
            Ok(uri) => uri,
            Err(err) => return err.once_err(),
        };

        uri.set_query(Some(&format!(
            "status={}&satelliteNames={}&start={}&end={}",
            status.as_ref(),
            account_uri.as_ref(),
            start.format(&Iso8601::DEFAULT).unwrap(),
            end.format(&Iso8601::DEFAULT).unwrap()
        )));

        self.get_paginated(uri)
    }

    /// Produces a vector of [`TaskRequest`] items, representing all the tasks which match the
    /// provided type, overlap with the provided time range.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    fn get_requests_by_type_and_target_date_between<T>(
        &self,
        typ: T,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> impl Future<Output = Result<Self::Container<Vec<TaskRequest>>, Error>> + Send + Sync
    where
        T: TryInto<TaskType> + Send + Sync,
        Error: From<<T as TryInto<TaskType>>::Error>,
    {
        async move {
            let typ: TaskType = typ.try_into()?;
            let mut uri = self.path_to_url("requests/search/findAllByTypeAndTargetDateBetween")?;

            uri.set_query(Some(&format!(
                "type={}&start={}&end={}",
                typ.as_ref(),
                start.format(&Iso8601::DEFAULT)?,
                end.format(&Iso8601::DEFAULT)?
            )));

            Ok(self
                .get_json_map::<Embedded<Self::Container<Vec<TaskRequest>>>>(uri)
                .await?
                .items)
        }
    }

    /// Produces a vector of [`TaskRequest`] items, representing the list of tasks which have
    /// already occurred today.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    fn get_requests_passed_today(
        &self,
    ) -> impl Future<Output = Result<Self::Container<Vec<TaskRequest>>, Error>> + Send + Sync {
        async move {
            let uri = self.path_to_url("requests/search/findAllPassedToday")?;

            Ok(self
                .get_json_map::<Embedded<Self::Container<Vec<TaskRequest>>>>(uri)
                .await?
                .items)
        }
    }

    /// Produces a vector of [`TaskRequest`] items, representing the list of tasks which will occur
    /// later today.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    fn get_requests_upcoming_today(
        &self,
    ) -> impl Future<Output = Result<Self::Container<Vec<TaskRequest>>, Error>> + Send + Sync {
        async move {
            let uri = self.path_to_url("requests/search/findAllUpcomingToday")?;

            Ok(self
                .get_json_map::<Embedded<Self::Container<Vec<TaskRequest>>>>(uri)
                .await?
                .items)
        }
    }

    /// Produces a paginated stream of [`Satellite`] objects.
    ///
    /// See [`get_paginated`](Self::get_paginated) documentation for more details about the process
    /// and return type
    fn get_satellites(&self) -> PaginatedStream<'_, Self::Container<Satellite>> {
        let uri = match self.path_to_url("satellites") {
            Ok(uri) => uri,
            Err(err) => return err.once_err(),
        };

        self.get_paginated(uri)
    }

    /// Produces single satellite object matching the provided satellite ID
    fn get_satellite_by_id(
        &self,
        satellite_id: i32,
    ) -> impl Future<Output = Result<Self::Container<Satellite>, Error>> + Send + Sync {
        async move {
            let uri = self.path_to_url(format!("satellites/{}", satellite_id))?;

            self.get_json_map(uri).await
        }
    }

    /// Produces single satellite object matching the provided satellite name
    fn get_satellite_by_name(
        &self,
        satellite_name: &str,
    ) -> impl Future<Output = Result<Self::Container<Satellite>, Error>> + Send + Sync {
        async move {
            let mut uri = self.path_to_url("satellites/findOneByName")?;
            uri.set_query(Some(&format!("name={satellite_name}")));

            self.get_json_map(uri).await
        }
    }

    /// Produces a single [`Task`] matching the provided ID.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    fn get_task_by_id(
        &self,
        task_id: i32,
    ) -> impl Future<Output = Result<Self::Container<Task>, Error>> + Send + Sync {
        async move {
            let uri = self.path_to_url(format!("tasks/{}", task_id))?;

            self.get_json_map(uri).await
        }
    }

    /// Produces a vector of [`Task`] items, representing all the tasks which match the provided
    /// account, and intersect with the provided time frame.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    fn get_tasks_by_account_and_pass_overlapping<T>(
        &self,
        account_uri: T,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> impl Future<Output = Result<Self::Container<Vec<Task>>, Error>> + Send + Sync
    where
        T: AsRef<str> + Send + Sync,
    {
        async move {
            let mut uri = self.path_to_url("tasks/search/findByAccountAndPassOverlapping")?;

            uri.set_query(Some(&format!(
                "account={}&start={}&end={}",
                account_uri.as_ref(),
                start.format(&Iso8601::DEFAULT)?,
                end.format(&Iso8601::DEFAULT)?
            )));

            Ok(self
                .get_json_map::<Embedded<Self::Container<Vec<Task>>>>(uri)
                .await?
                .items)
        }
    }

    /// Produces a vector of [`Task`] items, representing all the tasks which match the provided
    /// account, satellite, band, and intersect with the provided time frame.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    fn get_tasks_by_account_and_satellite_and_band_and_pass_overlapping<T, U, V>(
        &self,
        account_uri: T,
        satellite_config_uri: U,
        band: V,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> impl Future<Output = Result<Self::Container<Vec<Task>>, Error>> + Send + Sync
    where
        T: AsRef<str> + Send + Sync,
        U: AsRef<str> + Send + Sync,
        V: AsRef<str> + Send + Sync,
    {
        async move {
            let mut uri = self.path_to_url(
                "tasks/search/findByAccountAndSiteConfigurationAndBandAndPassOverlapping",
            )?;

            uri.set_query(Some(&format!(
                "account={}&satellite={}&band={}&start={}&end={}",
                account_uri.as_ref(),
                satellite_config_uri.as_ref(),
                band.as_ref(),
                start.format(&Iso8601::DEFAULT)?,
                end.format(&Iso8601::DEFAULT)?,
            )));

            Ok(self
                .get_json_map::<Embedded<Self::Container<Vec<Task>>>>(uri)
                .await?
                .items)
        }
    }

    /// Produces a vector of [`Task`] representing all the tasks which match the provided account,
    /// site configuration, band, and intersect with the provided time frame.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    fn get_tasks_by_account_and_site_configuration_and_band_and_pass_overlapping<T, U, V>(
        &self,
        account_uri: T,
        site_config_uri: U,
        band: V,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> impl Future<Output = Result<Self::Container<Vec<Task>>, Error>> + Send + Sync
    where
        T: AsRef<str> + Send + Sync,
        U: AsRef<str> + Send + Sync,
        V: AsRef<str> + Send + Sync,
    {
        async move {
            let mut uri = self.path_to_url(
                "tasks/search/findByAccountAndSiteConfigurationAndBandAndPassOverlapping",
            )?;

            uri.set_query(Some(&format!(
                "account={}&siteConfig={}&band={}&start={}&end={}",
                account_uri.as_ref(),
                site_config_uri.as_ref(),
                band.as_ref(),
                start.format(&Iso8601::DEFAULT)?,
                end.format(&Iso8601::DEFAULT)?
            )));

            Ok(self
                .get_json_map::<Embedded<Self::Container<Vec<Task>>>>(uri)
                .await?
                .items)
        }
    }

    /// Produces a vector of [`Task`] items, representing all the tasks contained within the
    /// provided time frame.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    ///
    /// # Note
    ///
    /// This differs from [`Self::get_tasks_by_pass_overlapping`] in that it only produces tasks
    /// which are wholly contained within the window.
    fn get_tasks_by_pass_window(
        &self,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> impl Future<Output = Result<Self::Container<Vec<Task>>, Error>> + Send + Sync {
        async move {
            let mut uri = self.path_to_url("tasks/search/findByStartBetweenOrderByStartAsc")?;

            uri.set_query(Some(&format!(
                "start={}&end={}",
                start.format(&Iso8601::DEFAULT)?,
                end.format(&Iso8601::DEFAULT)?
            )));

            Ok(self
                .get_json_map::<Embedded<Self::Container<Vec<Task>>>>(uri)
                .await?
                .items)
        }
    }

    /// Produces a paginated stream of [`Task`] items, representing all the tasks which overlap the
    /// provided time frame.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    ///
    /// # Note
    ///
    /// This differs from [`Self::get_tasks_by_pass_window`] in that it also includes tasks which
    /// only partially fall within the provided time frame.
    fn get_tasks_by_pass_overlapping(
        &self,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> PaginatedStream<'_, Self::Container<Task>> {
        let start = match start.format(&Iso8601::DEFAULT).map_err(Error::from) {
            Ok(start) => start,
            Err(error) => return error.once_err(),
        };

        let end = match end.format(&Iso8601::DEFAULT).map_err(Error::from) {
            Ok(end) => end,
            Err(error) => return error.once_err(),
        };

        let mut uri = match self.path_to_url("tasks/search/findByOverlapping") {
            Ok(uri) => uri,
            Err(err) => return err.once_err(),
        };

        uri.set_query(Some(&format!("start={}&end={}", start, end)));

        self.get_paginated(uri)
    }

    /// Produces a vector of [`Task`] items, representing the list of tasks which have already
    /// occurred today.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    fn get_tasks_passed_today(
        &self,
    ) -> impl Future<Output = Result<Self::Container<Vec<Task>>, Error>> + Send + Sync {
        async move {
            let uri = self.path_to_url("tasks/search/findAllPassedToday")?;

            Ok(self
                .get_json_map::<Embedded<Self::Container<Vec<Task>>>>(uri)
                .await?
                .items)
        }
    }

    /// Produces a vector of [`Task`] items, representing the list of tasks which will occur later
    /// today.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    fn get_tasks_upcoming_today(
        &self,
    ) -> impl Future<Output = Result<Self::Container<Vec<Task>>, Error>> + Send + Sync {
        async move {
            let uri = self.path_to_url("tasks/search/findAllUpcomingToday")?;

            Ok(self
                .get_json_map::<Embedded<Self::Container<Vec<Task>>>>(uri)
                .await?
                .items)
        }
    }

    /// Produces a paginated stream of [`User`] objects.
    ///
    /// See [`get_paginated`](Self::get_paginated) documentation for more details about the process
    /// and return type
    fn get_users(&self) -> PaginatedStream<'_, Self::Container<User>> {
        let uri = match self.path_to_url("users") {
            Ok(uri) => uri,
            Err(err) => return err.once_err(),
        };
        self.get_paginated(uri)
    }

    /// Create a new satellite band object
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use freedom_api::prelude::*;
    /// # tokio_test::block_on(async {
    /// let client = Client::from_env()?;
    ///
    /// client
    ///     .new_band_details()
    ///     .name("My Satellite Band")
    ///     .band_type(BandType::Receive)
    ///     .frequency(8096.0)
    ///     .default_band_width(1.45)
    ///     .io_hardware(IoHardware::Modem)
    ///     .send()
    ///     .await?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// # });
    /// ```
    fn new_band_details(&self) -> post::band::BandDetailsBuilder<'_, Self, post::band::NoName>
    where
        Self: Sized,
    {
        post::band::new(self)
    }

    /// Create a new satellite configuration
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use freedom_api::prelude::*;
    /// # tokio_test::block_on(async {
    /// let client = Client::from_env()?;
    ///
    /// client
    ///     .new_satellite_configuration()
    ///     .name("My Satellite Configuration")
    ///     .band_ids([1, 2, 3]) // List of band IDs to associate with config
    ///     .send()
    ///     .await?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// # });
    /// ```
    fn new_satellite_configuration(
        &self,
    ) -> post::sat_config::SatelliteConfigurationBuilder<'_, Self, post::sat_config::NoName>
    where
        Self: Sized,
    {
        post::sat_config::new(self)
    }

    /// Create a new satellite
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use freedom_api::prelude::*;
    /// # tokio_test::block_on(async {
    /// let client = Client::from_env()?;
    ///
    /// client
    ///     .new_satellite()
    ///     .name("My Satellite")
    ///     .satellite_configuration_id(42)
    ///     .norad_id(3600)
    ///     .description("A test satellite")
    ///     .send()
    ///     .await?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// # });
    /// ```
    fn new_satellite(&self) -> post::satellite::SatelliteBuilder<'_, Self, post::satellite::NoName>
    where
        Self: Sized,
    {
        post::satellite::new(self)
    }

    /// Create a new override
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use freedom_api::prelude::*;
    /// # tokio_test::block_on(async {
    /// let client = Client::from_env()?;
    ///
    /// client
    ///     .new_override()
    ///     .name("downconverter.gain override for sat 1 on config 2")
    ///     .satellite_id(1)
    ///     .satellite_configuration_id(2)
    ///     .add_property("site.hardware.modem.ttc.rx.demodulator.bitrate", 8096_u32)
    ///     .add_property("site.hardware.modem.ttc.tx.modulator.bitrate", 8096_u32)
    ///     .send()
    ///     .await?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// # });
    /// ```
    fn new_override(&self) -> post::overrides::OverrideBuilder<'_, Self, post::overrides::NoName>
    where
        Self: Sized,
    {
        post::overrides::new(self)
    }

    /// Create a new user
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use freedom_api::prelude::*;
    /// # tokio_test::block_on(async {
    /// let client = Client::from_env()?;
    ///
    /// client
    ///     .new_user()
    ///     .account_id(1)
    ///     .first_name("Han")
    ///     .last_name("Solo")
    ///     .email("flyingsolo@gmail.com")
    ///     .send()
    ///     .await?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// # });
    /// ```
    fn new_user(&self) -> post::user::UserBuilder<'_, Self, post::user::NoAccount>
    where
        Self: Sized,
    {
        post::user::new(self)
    }

    /// Create a new task request
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use freedom_api::prelude::*;
    /// # use time::OffsetDateTime;
    /// # use std::time::Duration;
    /// # tokio_test::block_on(async {
    /// let client = Client::from_env()?;
    ///
    /// client
    ///     .new_task_request()
    ///     .test_task("my_test_file.bin")
    ///     .target_time_utc(OffsetDateTime::now_utc() + Duration::from_secs(15 * 60))
    ///     .task_duration(120)
    ///     .satellite_id(1016)
    ///     .site_id(27)
    ///     .site_configuration_id(47)
    ///     .band_ids([2017, 2019])
    ///     .send()
    ///     .await?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// # });
    /// ```
    fn new_task_request(&self) -> post::TaskRequestBuilder<'_, Self, post::request::NoType>
    where
        Self: Sized,
    {
        post::request::new(self)
    }

    /// Fetch an FPS token for the provided band ID and site configuration ID
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use freedom_api::prelude::*;
    /// # tokio_test::block_on(async {
    /// const BAND_ID: u32 = 42;
    /// const SITE_CONFIG_ID: u32 = 201;
    ///
    /// let client = Client::from_env()?;
    ///
    /// let token = client.new_token_by_site_configuration_id(BAND_ID, SITE_CONFIG_ID).await?;
    /// // Submit token to FPS ...
    /// println!("{:?}", token);
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// # });
    /// ```
    fn new_token_by_site_configuration_id(
        &self,
        band_id: u32,
        site_configuration_id: u32,
    ) -> impl Future<Output = Result<String, Error>> + Send + Sync {
        async move {
            let url = self.path_to_url("fps")?;
            let payload = serde_json::json!({
                "band": format!("/api/satellite_bands/{}", band_id),
                "configuration": format!("/api/configurations/{}", site_configuration_id),
            });

            let value: JsonValue = self.post_json_map(url, &payload).await?;

            value
                .get("token")
                .ok_or(Error::Response(String::from("Missing token field")))?
                .as_str()
                .ok_or(Error::Response(String::from("Invalid type for token")))
                .map(|s| s.to_owned())
        }
    }

    /// Fetch an FPS token for the provided band ID and satellite ID
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use freedom_api::prelude::*;
    /// # tokio_test::block_on(async {
    /// const BAND_ID: u32 = 42;
    /// const SATELLITE_ID: u32 = 101;
    ///
    /// let client = Client::from_env()?;
    ///
    /// let token = client.new_token_by_satellite_id(BAND_ID, SATELLITE_ID).await?;
    /// // Submit token to FPS ...
    /// println!("{:?}", token);
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// # });
    /// ```
    fn new_token_by_satellite_id(
        &self,
        band_id: u32,
        satellite_id: u32,
    ) -> impl Future<Output = Result<String, Error>> + Send + Sync {
        async move {
            let url = self.path_to_url("fps")?;
            let payload = serde_json::json!({
                "band": format!("/api/satellite_bands/{}", band_id),
                "satellite": format!("/api/satellites/{}", satellite_id),
            });

            let value: JsonValue = self.post_json_map(url, &payload).await?;

            value
                .get("token")
                .ok_or(Error::Response(String::from("Missing token field")))?
                .as_str()
                .ok_or(Error::Response(String::from("Invalid type for token")))
                .map(|s| s.to_owned())
        }
    }
}

pub(crate) fn error_on_non_success(status: &StatusCode, body: &[u8]) -> Result<(), Error> {
    if !status.is_success() {
        return Err(Error::ResponseStatus {
            status: *status,
            error: String::from_utf8_lossy(body).to_string(),
        });
    }

    Ok(())
}
