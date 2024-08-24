//! # ATLAS Freedom API
//!
//! This module exists to define the Freedom API trait, which can be implemented for multiple client
//! types.
//!
//! The API trait
#![allow(clippy::type_complexity)]
use std::{ops::Deref, pin::Pin};

use async_stream::stream;
use async_trait::async_trait;
use bytes::Bytes;
use freedom_config::Config;
use freedom_models::{
    account::Account,
    band::Band,
    pagination::Paginated,
    satellite::Satellite,
    satellite_configuration::SatelliteConfiguration,
    site::Site,
    task::{Task, TaskRequest, TaskStatusType, TaskType},
    user::User,
    utils::Embedded,
};
use reqwest::{Response, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use time::{format_description::well_known::Iso8601, OffsetDateTime};
use url::Url;

use futures_core::Stream;

use crate::{error::Error, prelude::RuntimeError};

pub(crate) mod post;

/// A super trait containing all the requirements for Freedom API Values
pub trait FreedomApiValue: std::fmt::Debug + DeserializeOwned + Clone + Send + Sync {}

impl<T> FreedomApiValue for T where T: std::fmt::Debug + DeserializeOwned + Clone + Send + Sync {}

trait PaginatedErr<'a, T> {
    fn once_err(self) -> PaginatedStream<'a, T>;
}

impl<'a, T: 'a + Send> PaginatedErr<'a, T> for Error {
    fn once_err(self) -> PaginatedStream<'a, T> {
        Box::pin(async_stream::stream! { yield Err(self); })
    }
}

/// The trait defining the required functionality container types
///
/// The Freedom API is generic over "containers". Each implementer of the [`FreedomApi`] trait must
/// also define a container. This is useful since certain clients will return Arc'd values, i.e. the
/// caching client. While others return the values wrapped in a simple `Inner` type which is just
/// a stack value
pub trait FreedomApiContainer<T>: Deref<Target = T> + FreedomApiValue {
    fn into_inner(self) -> T;
}

/// A stream of paginated results from freedom.
///
/// Each item in the stream is a result, since one or more items may fail to be serialized
pub type PaginatedStream<'a, T> = Pin<Box<dyn Stream<Item = Result<T, Error>> + 'a + Send>>;

/// The primary trait for interfacing with Freedom
#[async_trait]
pub trait FreedomApi: Send + Sync {
    /// The [`FreedomApi`] supports implementors with different so-called "container" types.
    ///
    /// Certain [`FreedomApi`] clients return an `Arc<T>` for each call, others return an `Inner<T>`
    /// which is a simple wrapper for a stack value.
    type Container<T: FreedomApiValue>: FreedomApiContainer<T>;

    /// Creates a get request at the provided absolute URI for the client's environment, using basic
    /// authentication.
    ///
    /// The JSON response is then deserialized into the required type, erroring if the
    /// deserialization fails, and providing the object if it succeeds.
    async fn get_json_map<T>(&self, url: Url) -> Result<T, Error>
    where
        T: FreedomApiValue,
    {
        let (body, status) = self.get(url).await?;

        error_on_non_success(&status)?;

        let utf8_str = String::from_utf8_lossy(&body);
        serde_json::from_str(&utf8_str).map_err(From::from)
    }

    /// Creates a get request at the provided absolute URI for the client's environment, using basic
    /// authentication.
    ///
    /// Returns the raw binary body, and the status code.
    async fn get(&self, url: Url) -> Result<(Bytes, StatusCode), Error>;

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
        T: 'static + FreedomApiValue,
    {
        let base = self.config().environment().freedom_entrypoint();
        let mut current_url = head_url; // Not necessary but makes control flow more obvious
        Box::pin(stream! {
            loop {
                // Get the results for the current page.
                let pag = self.get_json_map::<Paginated<serde_json::Value>>(current_url).await?;
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

    fn config(&self) -> &Config;

    fn config_mut(&mut self) -> &mut Config;

    /// Fetch the URL from the given path
    ///
    /// # Panics
    ///
    /// Panics in the event the URL cannot be constructed from the provided path
    fn path_to_url(&self, path: impl AsRef<str>) -> Url {
        let url = self.config().environment().freedom_entrypoint();
        url.join(path.as_ref()).expect("Invalid URL construction")
    }

    async fn delete(&self, url: Url) -> Result<Response, Error>;

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
    async fn delete_band_details(&self, id: i32) -> Result<Response, Error> {
        let uri = self.path_to_url(format!("satellite_bands/{id}"));
        self.delete(uri).await
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
    async fn delete_satellite_configuration(&self, id: i32) -> Result<Response, Error> {
        let uri = self.path_to_url(format!("satellite_configurations/{id}"));
        self.delete(uri).await
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
    async fn delete_satellite(&self, id: i32) -> Result<Response, Error> {
        let uri = self.path_to_url(format!("satellites/{id}"));
        self.delete(uri).await
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
    async fn delete_override(&self, id: i32) -> Result<Response, Error> {
        let uri = self.path_to_url(format!("overrides/{id}"));
        self.delete(uri).await
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
    async fn delete_user(&self, id: i32) -> Result<Response, Error> {
        let uri = self.path_to_url(format!("users/{id}"));
        self.delete(uri).await
    }

    /// Request to delete the user matching the provided `id`
    async fn delete_task_request(&self, id: i32) -> Result<Response, Error> {
        let uri = self.path_to_url(format!("requests/{id}"));
        self.delete(uri).await
    }

    async fn post_deserialize<S, T>(&self, url: Url, msg: S) -> Result<T, Error>
    where
        S: serde::Serialize + Send + Sync,
        T: FreedomApiValue,
    {
        let resp = self.post(url, msg).await?;

        resp.json::<T>().await.map_err(From::from)
    }

    async fn post<S>(&self, url: Url, msg: S) -> Result<Response, Error>
    where
        S: serde::Serialize + Send + Sync;

    /// Produces a single [`Account`](freedom_models::account::Account) matching the provided ID.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    async fn get_account_by_name(
        &self,
        account_name: &str,
    ) -> Result<Self::Container<Account>, Error> {
        let mut uri = self.path_to_url("accounts/search/findOneByName");
        uri.set_query(Some(&format!("name={account_name}")));
        self.get_json_map(uri).await
    }

    /// Produces a single [`Account`](freedom_models::account::Account) matching the provided ID.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    async fn get_file_by_task_id_and_name(
        &self,
        task_id: i32,
        file_name: &str,
    ) -> Result<Bytes, Error> {
        let path = format!("downloads/{}/{}", task_id, file_name);
        let uri = self.path_to_url(path);

        let (data, status) = self.get(uri).await?;
        error_on_non_success(&status)?;

        Ok(data)
    }

    /// Create a new satellite band object
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use freedom_api::prelude::*;
    /// # use freedom_models::band::{BandType, IoHardware};
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

    /// Produces a single [`Account`](freedom_models::account::Account) matching the provided ID.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    async fn get_account_by_id(&self, account_id: i32) -> Result<Self::Container<Account>, Error> {
        let uri = self.path_to_url(format!("accounts/{account_id}"));
        self.get_json_map(uri).await
    }

    /// Produces a paginated stream of [`Account`](freedom_models::account::Account) objects.
    ///
    /// See [`get_paginated`](Self::get_paginated) documentation for more details about the process
    /// and return type
    fn get_accounts(
        &self,
    ) -> Pin<Box<dyn Stream<Item = Result<Self::Container<Account>, Error>> + '_>> {
        let uri = self.path_to_url("accounts");
        self.get_paginated(uri)
    }

    /// Produces a paginated stream of [`Band`] objects.
    ///
    /// See [`get_paginated`](Self::get_paginated) documentation for more details about the process
    /// and return type
    fn get_satellite_bands(
        &self,
    ) -> Pin<Box<dyn Stream<Item = Result<Self::Container<Band>, Error>> + '_>> {
        let uri = self.path_to_url("satellite_bands/search/findAll");
        self.get_paginated(uri)
    }

    /// Produces a single [`Band`] matching the provided ID.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    async fn get_satellite_band_by_id(
        &self,
        satellite_band_id: i32,
    ) -> Result<Self::Container<Band>, Error> {
        let uri = self.path_to_url(format!("satellite_bands/{satellite_band_id}"));
        self.get_json_map(uri).await
    }

    /// Produces a single [`Band`] matching the provided name.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    async fn get_satellite_band_by_name(
        &self,
        satellite_band_name: &str,
    ) -> Result<Self::Container<Band>, Error> {
        let mut uri = self.path_to_url("satellite_bands/search/findOneByName");
        uri.set_query(Some(&format!("name={satellite_band_name}")));
        self.get_json_map(uri).await
    }

    /// Produces a paginated stream of [`Band`] objects matching the provided account name.
    ///
    /// See [`get_paginated`](Self::get_paginated) documentation for more details about the process
    /// and return type
    fn get_satellite_bands_by_account_name(
        &self,
        account_name: &str,
    ) -> Pin<Box<dyn Stream<Item = Result<Self::Container<Band>, Error>> + '_>> {
        let mut uri = self.path_to_url("satellite_bands/search/findAllByAccountName");
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
    ) -> Pin<Box<dyn Stream<Item = Result<Self::Container<SatelliteConfiguration>, Error>> + '_>>
    {
        let mut uri = self.path_to_url("satellite_configurations/search/findAllByAccountName");
        uri.set_query(Some(&format!("accountName={account_name}")));

        self.get_paginated(uri)
    }

    /// Produces a paginated stream of [`SatelliteConfiguration`] objects.
    ///
    /// See [`get_paginated`](Self::get_paginated) documentation for more details about the process
    /// and return type
    fn get_satellite_configurations(
        &self,
    ) -> Pin<Box<dyn Stream<Item = Result<Self::Container<SatelliteConfiguration>, Error>> + '_>>
    {
        let uri = self.path_to_url("satellite_configurations/search/findAll");

        self.get_paginated(uri)
    }

    /// Produces a paginated stream of [`SatelliteConfiguration`] objects.
    ///
    /// See [`get_paginated`](Self::get_paginated) documentation for more details about the process
    /// and return type
    fn get_satellite_configurations_by_id(
        &self,
        satellite_configuration_id: i32,
    ) -> Pin<Box<dyn Stream<Item = Result<Self::Container<SatelliteConfiguration>, Error>> + '_>>
    {
        let uri = self.path_to_url(format!(
            "satellite_configurations/{satellite_configuration_id}"
        ));

        self.get_paginated(uri)
    }

    /// Produces a paginated stream of [`Site`] objects.
    ///
    /// See [`get_paginated`](Self::get_paginated) documentation for more details about the process
    /// and return type
    fn get_sites(&self) -> Pin<Box<dyn Stream<Item = Result<Self::Container<Site>, Error>> + '_>> {
        let uri = self.path_to_url("sites");
        self.get_paginated(uri)
    }

    /// Produces a single [`Site`] object matching the provided ID.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    async fn get_site_by_id(&self, id: i32) -> Result<Self::Container<Site>, Error> {
        let uri = self.path_to_url(format!("sites/{id}"));
        self.get_json_map(uri).await
    }

    /// Produces a single [`Site`] object matching the provided name.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    async fn get_site_by_name(
        &self,
        name: impl AsRef<str> + Send,
    ) -> Result<Self::Container<Site>, Error> {
        let mut uri = self.path_to_url("sites/search/findOneByName");
        let query = format!("name={}", name.as_ref());
        uri.set_query(Some(&query));

        self.get_json_map(uri).await
    }

    /// Produces a single [`TaskRequest`] matching the provided ID.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    async fn get_request_by_id(
        &self,
        task_request_id: i32,
    ) -> Result<Self::Container<TaskRequest>, Error> {
        let uri = self.path_to_url(format!("requests/{task_request_id}"));

        self.get_json_map(uri).await
    }

    /// Produces a paginated stream of [`TaskRequest`] objects.
    ///
    /// See [`get_paginated`](Self::get_paginated) documentation for more details about the process
    /// and return type
    fn get_requests(&self) -> PaginatedStream<'_, Self::Container<TaskRequest>> {
        {
            let uri = self.path_to_url("requests/search/findAll");
            self.get_paginated(uri)
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
        T: AsRef<str> + Send,
    {
        let mut uri = self.path_to_url("requests/search/findAllByAccountAndTargetDateBetween");

        uri.set_query(Some(&format!(
            "account={}&start={}&end={}",
            account_uri.as_ref(),
            start.format(&Iso8601::DEFAULT).unwrap(),
            end.format(&Iso8601::DEFAULT).unwrap(),
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
        let uri = self.path_to_url("requests/search/findByAccountUpcomingToday");

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
        T: AsRef<str> + Send,
    {
        let mut uri = self.path_to_url("requests/search/findAllByConfigurationOrderByCreatedAsc");

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
    async fn get_requests_by_configuration_and_satellite_names_and_target_date_between<T, I, S>(
        &self,
        configuration_uri: T,
        satellites: I,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> Result<Self::Container<Vec<TaskRequest>>, Error>
    where
        T: AsRef<str> + Send,
        I: IntoIterator<Item = S> + Send,
        S: AsRef<str> + Send,
    {
        let satellites_string = crate::utils::list_to_string(satellites);
        let mut uri = self.path_to_url(
            "requests/search/findAllByConfigurationAndSatelliteNamesAndTargetDateBetween",
        );

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

    /// Produces a vector of [`TaskRequest`] items, representing all the task requests matching the
    /// configuration at the provided URI and whose target time overlaps with the provided time
    /// range.
    ///
    /// See [`get_paginated`](Self::get_paginated) documentation for more details about the process
    /// and return type
    async fn get_requests_by_configuration_and_target_date_between<T>(
        &self,
        configuration_uri: T,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> Result<Self::Container<Vec<TaskRequest>>, Error>
    where
        T: AsRef<str> + Send,
    {
        let mut uri =
            self.path_to_url("requests/search/findAllByConfigurationAndTargetDateBetween");
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

    /// Produces a vector of [`TaskRequest`] items,
    /// representing all the task requests whose ID matches one of the IDs provided as part of
    /// `ids`.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    async fn get_requests_by_ids<I, S>(
        &self,
        ids: I,
    ) -> Result<Self::Container<Vec<TaskRequest>>, Error>
    where
        I: IntoIterator<Item = S> + Send,
        S: AsRef<str> + Send,
    {
        let ids_string = crate::utils::list_to_string(ids);
        let mut uri = self.path_to_url("requests/search/findAllByIds");

        uri.set_query(Some(&format!("ids={}", ids_string)));

        Ok(self
            .get_json_map::<Embedded<Self::Container<Vec<TaskRequest>>>>(uri)
            .await?
            .items)
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
        let mut uri = self.path_to_url("requests/search/findAllByOverlappingPublic");

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
        T: AsRef<str> + Send,
    {
        let mut uri = self.path_to_url("requests/search/findBySatelliteName");

        uri.set_query(Some(&format!("name={}", satellite_name.as_ref())));

        self.get_paginated(uri)
    }

    /// Produces a vector of [`TaskRequest`] items, representing all the task requests whose
    /// satellite name matches the provided name and whose target time overlaps with the provided
    /// time range.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    async fn get_requests_by_satellite_name_and_target_date_between<T>(
        &self,
        satellite_name: T,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> Result<Self::Container<Vec<TaskRequest>>, Error>
    where
        T: AsRef<str> + Send,
    {
        let mut uri =
            self.path_to_url("requests/search/findAllBySatelliteNameAndTargetDateBetween");

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

    /// Produces a paginated stream of [`TaskRequest`] objects whose status matches the provided
    /// status.
    ///
    /// See [`get_paginated`](Self::get_paginated) documentation for more details about the process
    /// and return type
    fn get_requests_by_status<T>(
        &self,
        status: T,
    ) -> Result<PaginatedStream<'_, Self::Container<TaskRequest>>, Error>
    where
        T: TryInto<TaskStatusType> + Send,
        Error: From<<T as TryInto<TaskStatusType>>::Error>,
    {
        let status: TaskStatusType = status.try_into()?;
        let mut uri = self.path_to_url("requests/search/findByStatus");

        uri.set_query(Some(&format!("status={}", status.as_ref())));

        Ok(self.get_paginated(uri))
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
        T: AsRef<str> + Send,
        U: AsRef<str> + Send,
    {
        let mut uri =
            self.path_to_url("requests/search/findAllByStatusAndAccountAndTargetDateBetween");

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
    async fn get_requests_by_type_and_target_date_between<T>(
        &self,
        typ: T,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> Result<Self::Container<Vec<TaskRequest>>, Error>
    where
        T: TryInto<TaskType> + Send,
        Error: From<<T as TryInto<TaskType>>::Error>,
    {
        let typ: TaskType = typ.try_into()?;
        let mut uri = self.path_to_url("requests/search/findAllByTypeAndTargetDateBetween");

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

    /// Produces a vector of [`TaskRequest`] items, representing the list of tasks which have
    /// already occurred today.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    async fn get_requests_passed_today(&self) -> Result<Self::Container<Vec<TaskRequest>>, Error> {
        let uri = self.path_to_url("requests/search/findAllPassedToday");

        Ok(self
            .get_json_map::<Embedded<Self::Container<Vec<TaskRequest>>>>(uri)
            .await?
            .items)
    }

    /// Produces a vector of [`TaskRequest`] items, representing the list of tasks which will occur
    /// later today.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    async fn get_requests_upcoming_today(
        &self,
    ) -> Result<Self::Container<Vec<TaskRequest>>, Error> {
        let uri = self.path_to_url("requests/search/findAllUpcomingToday");

        Ok(self
            .get_json_map::<Embedded<Self::Container<Vec<TaskRequest>>>>(uri)
            .await?
            .items)
    }

    /// Produces a paginated stream of [`Satellite`] objects.
    ///
    /// See [`get_paginated`](Self::get_paginated) documentation for more details about the process
    /// and return type
    fn get_satellites(&self) -> PaginatedStream<'_, Self::Container<Satellite>> {
        let uri = self.path_to_url("satellites");

        self.get_paginated(uri)
    }

    /// Produces a single [`Task`] matching the provided ID.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    async fn get_task_by_id(&self, task_id: i32) -> Result<Self::Container<Task>, Error> {
        let uri = self.path_to_url(format!("tasks/{}", task_id));

        self.get_json_map(uri).await
    }

    /// Produces a vector of [`Task`] items, representing all the tasks which match the provided
    /// account, and intersect with the provided time frame.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    async fn get_tasks_by_account_and_pass_overlapping<T>(
        &self,
        account_uri: T,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> Result<Self::Container<Vec<Task>>, Error>
    where
        T: AsRef<str> + Send,
    {
        let mut uri = self.path_to_url("tasks/search/findByAccountAndPassOverlapping");

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

    /// Produces a vector of [`Task`] items, representing all the tasks which match the provided
    /// account, satellite, band, and intersect with the provided time frame.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    async fn get_tasks_by_account_and_satellite_and_band_and_pass_overlapping<T, U, V>(
        &self,
        account_uri: T,
        satellite_config_uri: U,
        band: V,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> Result<Self::Container<Vec<Task>>, Error>
    where
        T: AsRef<str> + Send,
        U: AsRef<str> + Send,
        V: AsRef<str> + Send,
    {
        let mut uri = self
            .path_to_url("tasks/search/findByAccountAndSiteConfigurationAndBandAndPassOverlapping");

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

    /// Produces a vector of [`Task`] representing all the tasks which match the provided account,
    /// site configuration, band, and intersect with the provided time frame.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    async fn get_tasks_by_account_and_site_configuration_and_band_and_pass_overlapping<T, U, V>(
        &self,
        account_uri: T,
        site_config_uri: U,
        band: V,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> Result<Self::Container<Vec<Task>>, Error>
    where
        T: AsRef<str> + Send,
        U: AsRef<str> + Send,
        V: AsRef<str> + Send,
    {
        let mut uri = self
            .path_to_url("tasks/search/findByAccountAndSiteConfigurationAndBandAndPassOverlapping");

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

    /// Produces a vector of [`Task`] items, representing all the tasks contained within the
    /// provided time frame.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    ///
    /// # Note
    ///
    /// This differs from [`Self::get_tasks_by_pass_overlapping`] in that it only produces tasks
    /// which are wholly contained within the window.
    async fn get_tasks_by_pass_window(
        &self,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> Result<Self::Container<Vec<Task>>, Error> {
        let mut uri = self.path_to_url("tasks/search/findByStartBetweenOrderByStartAsc");

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

        let mut uri = self.path_to_url("tasks/search/findByOverlapping");

        uri.set_query(Some(&format!("start={}&end={}", start, end)));

        self.get_paginated(uri)
    }

    /// Produces a vector of [`Task`] items, representing the list of tasks which have already
    /// occurred today.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    async fn get_tasks_passed_today(&self) -> Result<Self::Container<Vec<Task>>, Error> {
        let uri = self.path_to_url("tasks/search/findAllPassedToday");

        Ok(self
            .get_json_map::<Embedded<Self::Container<Vec<Task>>>>(uri)
            .await?
            .items)
    }

    /// Produces a vector of [`Task`] items, representing the list of tasks which will occur later
    /// today.
    ///
    /// See [`get`](Self::get) documentation for more details about the process and return type
    async fn get_tasks_upcoming_today(&self) -> Result<Self::Container<Vec<Task>>, Error> {
        let uri = self.path_to_url("tasks/search/findAllUpcomingToday");

        Ok(self
            .get_json_map::<Embedded<Self::Container<Vec<Task>>>>(uri)
            .await?
            .items)
    }

    /// Fetch a token by providing a POST value
    ///
    /// # Warning
    ///
    /// Do not use this method, directly. Instead prefer [`FreedomApi::new_token_by_satellite_id`] or
    /// [`FreedomApi::new_token_by_site_configuration_id`]
    async fn new_token<S: std::fmt::Debug + Serialize + Sync + Send>(
        &self,
        post_val: S,
    ) -> Result<String, Error> {
        let uri = self.path_to_url("fps");

        let value: Value = self.post_deserialize(uri, post_val).await?;

        value
            .get("token")
            .ok_or(RuntimeError::Response(String::from("Missing token field")))?
            .as_str()
            .ok_or(RuntimeError::Response(String::from(
                "Invalid type for token",
            )))
            .map(|s| s.to_owned())
            .map_err(From::from)
    }

    /// Fetch an FPS token for the provided band ID and site configuration ID
    async fn new_token_by_site_configuration_id(
        &self,
        band_id: u32,
        site_configuration_id: u32,
    ) -> Result<String, crate::Error> {
        let payload = serde_json::json!({
            "band": format!("/api/satellite_bands/{}", band_id),
            "configuration": format!("/api/configurations/{}", site_configuration_id),
        });

        self.new_token(&payload).await
    }

    /// Fetch an FPS token for the provided band ID and satellite ID
    async fn new_token_by_satellite_id(
        &self,
        band_id: u32,
        satellite_id: u32,
    ) -> Result<String, crate::Error> {
        let payload = serde_json::json!({
            "band": format!("/api/satellite_bands/{}", band_id),
            "satellite": format!("/api/satellites/{}", satellite_id),
        });

        self.new_token(&payload).await
    }

    /// Produces a paginated stream of [`User`] objects.
    ///
    /// See [`get_paginated`](Self::get_paginated) documentation for more details about the process
    /// and return type
    fn get_users(&self) -> Pin<Box<dyn Stream<Item = Result<Self::Container<User>, Error>> + '_>> {
        let uri = self.path_to_url("users");
        self.get_paginated(uri)
    }
}

fn error_on_non_success(status: &StatusCode) -> Result<(), Error> {
    if !status.is_success() {
        return Err(Error::Runtime(RuntimeError::Response(status.to_string())));
    }

    Ok(())
}
