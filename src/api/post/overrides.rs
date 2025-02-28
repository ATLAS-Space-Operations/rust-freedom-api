use std::collections::HashMap;

use reqwest::Response;
use serde::Serialize;

use crate::{api::Api, error::Error};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Override {
    name: String,
    satellite: String,
    configuration: String,
    properties: HashMap<String, String>,
}

pub struct OverrideBuilder<'a, C, S> {
    pub(crate) client: &'a C,
    state: S,
}

pub fn new<C>(client: &C) -> OverrideBuilder<'_, C, NoName> {
    OverrideBuilder {
        client,
        state: NoName,
    }
}

pub struct NoName;

impl<'a, C> OverrideBuilder<'a, C, NoName> {
    pub fn name(self, name: impl Into<String>) -> OverrideBuilder<'a, C, NoSatellite> {
        OverrideBuilder {
            client: self.client,
            state: NoSatellite { name: name.into() },
        }
    }
}

pub struct NoSatellite {
    name: String,
}

impl<'a, C> OverrideBuilder<'a, C, NoSatellite>
where
    C: Api,
{
    pub fn satellite_id(self, id: impl Into<i32>) -> OverrideBuilder<'a, C, NoConfig> {
        let satellite = self
            .client
            .path_to_url(format!("satellites/{}", id.into()))
            .to_string();

        self.satellite_url(satellite)
    }
}

impl<'a, C> OverrideBuilder<'a, C, NoSatellite> {
    pub fn satellite_url(self, url: impl Into<String>) -> OverrideBuilder<'a, C, NoConfig> {
        OverrideBuilder {
            client: self.client,
            state: NoConfig {
                name: self.state.name,
                satellite: url.into(),
            },
        }
    }
}

pub struct NoConfig {
    name: String,
    satellite: String,
}

impl<'a, C> OverrideBuilder<'a, C, NoConfig>
where
    C: Api,
{
    pub fn satellite_configuration_id(
        self,
        id: impl Into<i32>,
    ) -> OverrideBuilder<'a, C, Override> {
        let satellite = self
            .client
            .path_to_url(format!("satellites/{}", id.into()))
            .to_string();

        self.satellite_configuration_url(satellite)
    }
}

impl<'a, C> OverrideBuilder<'a, C, NoConfig> {
    pub fn satellite_configuration_url(
        self,
        url: impl Into<String>,
    ) -> OverrideBuilder<'a, C, Override> {
        let state = Override {
            name: self.state.name,
            satellite: self.state.satellite,
            configuration: url.into(),
            properties: HashMap::new(),
        };

        OverrideBuilder {
            client: self.client,
            state,
        }
    }
}

impl<C> OverrideBuilder<'_, C, Override> {
    pub fn add_property(mut self, key: impl Into<String>, value: impl ToString) -> Self {
        self.state.properties.insert(key.into(), value.to_string());
        self
    }
}

impl<C> OverrideBuilder<'_, C, Override>
where
    C: Api,
{
    pub async fn send(self) -> Result<Response, Error> {
        let client = self.client;

        let url = client.path_to_url("overrides");
        client.post(url, self.state).await
    }
}
