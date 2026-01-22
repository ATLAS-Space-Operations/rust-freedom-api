pub mod band;
pub mod overrides;
pub mod request;
pub mod sat_config;
pub mod satellite;
pub mod user;

pub use self::{
    band::BandDetailsBuilder, overrides::OverrideBuilder, request::TaskRequestBuilder,
    sat_config::SatelliteConfigurationBuilder, satellite::SatelliteBuilder, user::UserBuilder,
};

use super::Value;

/// A trait defining the response and send future for HTTP POSTs
pub trait Post {
    /// The deserialized response from the Freedom API after a successful POST
    type Response: Value;

    /// The future used to send the POST
    fn send(
        self,
    ) -> impl Future<Output = Result<Self::Response, crate::error::Error>> + Send + Sync;
}

#[derive(Debug, Clone, PartialEq)]
enum UrlResult {
    Checked(String),
    Unchecked(Result<url::Url, crate::error::Error>),
}

impl UrlResult {
    fn try_convert(self) -> Result<String, crate::error::Error> {
        match self {
            UrlResult::Checked(url) => Ok(url),
            UrlResult::Unchecked(Ok(url)) => Ok(url.to_string()),
            UrlResult::Unchecked(Err(error)) => Err(error),
        }
    }
}
