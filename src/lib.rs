#![doc = include_str!("../README.md")]

mod api;
#[cfg(feature = "caching")]
mod caching_client;
mod client;
pub mod error;
pub mod extensions;
mod utils;

/// Result type for the API
pub type Result<T> = std::result::Result<T, Error>;

pub use self::{
    api::{Api, Container, Value},
    client::Client,
    error::Error,
};

/// Contains the client, data models, and error types necessary for queries
pub mod prelude {
    #[cfg(feature = "caching")]
    pub use crate::caching_client::CachingClient;
    pub use crate::{
        api::{
            post::{
                BandDetailsBuilder, OverrideBuilder, SatelliteBuilder,
                SatelliteConfigurationBuilder, UserBuilder,
            },
            Api, Container, Value,
        },
        client::Client,
        error::Error,
        extensions::*,
        models::*,
    };
}

/// Data type exposed by the Freedom API
///
/// Re-export of the models found in the `freedom-models` crate.
pub mod models {
    pub use freedom_models::{
        account::*, azel::*, band::*, satellite::*, satellite_configuration::*, site::*, task::*,
        user::*,
    };
}
