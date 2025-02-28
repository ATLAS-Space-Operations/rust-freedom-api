use reqwest::Response;
use serde::Serialize;

use crate::{api::Api, error::Error};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Satellite {
    name: String,
    description: Option<String>,
    norad_cat_id: u32,
    configuration: String,
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
    pub fn satellite_configuration_url(
        self,
        url: impl Into<String>,
    ) -> SatelliteBuilder<'a, C, NoNorad> {
        SatelliteBuilder {
            client: self.client,
            state: NoNorad {
                name: self.state.name,
                configuration: url.into(),
            },
        }
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
            .path_to_url(format!("satellite_configurations/{}", id.into()))
            .to_string();

        self.satellite_configuration_url(configuration)
    }
}

pub struct NoNorad {
    name: String,
    configuration: String,
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
    pub async fn send(self) -> Result<Response, Error> {
        let client = self.client;

        let url = client.path_to_url("satellites");
        client.post(url, self.state).await
    }
}
