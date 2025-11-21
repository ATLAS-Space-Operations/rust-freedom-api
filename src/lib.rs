#![doc = include_str!("../README.md")]

mod api;
#[cfg(feature = "caching")]
mod caching_client;
mod client;
pub mod error;
pub mod extensions;
mod utils;

#[cfg(feature = "bundles")]
pub use self::api::bundle::BundleApi;
pub use self::{
    api::{Api, Container, Inner, PaginatedStream, Value},
    client::Client,
};

/// Contains the client, data models, and traits necessary for queries
pub mod prelude {
    #[cfg(feature = "caching")]
    pub use crate::caching_client::CachingClient;
    pub use crate::{
        api::{
            Api, Container, Inner, PaginatedStream, Value,
            post::{
                BandDetailsBuilder, OverrideBuilder, SatelliteBuilder,
                SatelliteConfigurationBuilder, UserBuilder,
            },
        },
        client::Client,
        config::*,
        extensions::*,
        models::*,
    };
}

/// Data types exposed by the Freedom API
///
/// Re-export of the models found in the `freedom-models` crate.
pub mod models {
    pub use freedom_models::{
        account::*, azel::*, band::*, satellite::*, satellite_configuration::*, site::*, task::*,
        user::*,
    };
}

/// Configuration options for Freedom API
///
/// Re-export of the types found in the `freedom-config` crate.
pub mod config {
    pub use freedom_config::{
        Config, ConfigBuilder, Env, Environment, IntoEnv, Prod, Secret, Test,
    };
}
