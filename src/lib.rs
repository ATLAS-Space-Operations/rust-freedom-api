#![doc = include_str!("../README.md")]

mod api;
#[cfg(feature = "caching")]
mod caching_client;
mod client;
pub mod error;
pub mod extensions;
mod utils;

pub use client::Client;
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

/// Contains the client, data models, and error types necessary for queries
pub mod prelude {
    pub use crate::api::post::{
        BandDetailsBuilder, OverrideBuilder, SatelliteBuilder, SatelliteConfigurationBuilder,
    };
    pub use crate::api::{FreedomApi, FreedomApiContainer, FreedomApiValue};
    #[cfg(feature = "caching")]
    pub use crate::caching_client::CachingClient;
    pub use crate::extensions::*;
    pub use crate::{
        client::Client,
        error::{BuilderError, Error, RuntimeError},
    };
    pub use freedom_models::{
        account::*, satellite::*, satellite_configuration::*, site::*, task::*, user::*,
    };
}

pub use freedom_models::{account::*, satellite::*, site::*, task::*, user::*};
