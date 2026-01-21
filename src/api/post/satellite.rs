use serde::Serialize;

use crate::{api::Api, error::Error};

use super::UrlResult;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct SatelliteInner {
    name: String,
    description: Option<String>,
    norad_cat_id: u32,
    configuration: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Satellite {
    name: String,
    description: Option<String>,
    norad_cat_id: u32,
    configuration: UrlResult,
}

pub fn new<C>(client: &C) -> SatelliteBuilder<'_, C, NoName> {
    SatelliteBuilder {
        client,
        state: NoName,
    }
}

pub struct SatelliteBuilder<'a, C, S> {
    pub(crate) client: &'a C,
    state: S,
}

pub struct NoName;

impl<'a, C> SatelliteBuilder<'a, C, NoName> {
    pub fn name(self, name: impl Into<String>) -> SatelliteBuilder<'a, C, NoConfig> {
        SatelliteBuilder {
            client: self.client,
            state: NoConfig { name: name.into() },
        }
    }
}

pub struct NoConfig {
    name: String,
}

impl<'a, C> SatelliteBuilder<'a, C, NoConfig> {
    fn satellite_configuration_result(self, url: UrlResult) -> SatelliteBuilder<'a, C, NoNorad> {
        SatelliteBuilder {
            client: self.client,
            state: NoNorad {
                name: self.state.name,
                configuration: url,
            },
        }
    }

    pub fn satellite_configuration_url(
        self,
        url: impl Into<String>,
    ) -> SatelliteBuilder<'a, C, NoNorad> {
        self.satellite_configuration_result(UrlResult::Checked(url.into()))
    }
}

impl<'a, C> SatelliteBuilder<'a, C, NoConfig>
where
    C: Api,
{
    pub fn satellite_configuration_id(
        self,
        id: impl Into<i32>,
    ) -> SatelliteBuilder<'a, C, NoNorad> {
        let configuration = self
            .client
            .path_to_url(format!("satellite_configurations/{}", id.into()));

        self.satellite_configuration_result(UrlResult::Unchecked(configuration))
    }
}

pub struct NoNorad {
    name: String,
    configuration: UrlResult,
}

impl<'a, C> SatelliteBuilder<'a, C, NoNorad> {
    pub fn norad_id(self, norad_id: u32) -> SatelliteBuilder<'a, C, Satellite> {
        let state = Satellite {
            name: self.state.name,
            description: None,
            norad_cat_id: norad_id,
            configuration: self.state.configuration,
        };

        SatelliteBuilder {
            client: self.client,
            state,
        }
    }
}

impl<C> SatelliteBuilder<'_, C, Satellite> {
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.state.description = Some(description.into());
        self
    }
}

impl<C> SatelliteBuilder<'_, C, Satellite>
where
    C: Api,
{
    pub async fn send(self) -> Result<freedom_models::satellite::Satellite, Error> {
        let client = self.client;

        let url = client.path_to_url("satellites")?;
        let dto = SatelliteInner {
            name: self.state.name,
            description: self.state.description,
            norad_cat_id: self.state.norad_cat_id,
            configuration: self.state.configuration.try_convert()?,
        };

        client.post_json_map(url, dto).await
    }
}
