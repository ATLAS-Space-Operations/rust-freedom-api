use std::collections::HashMap;

use reqwest::Response;
use serde::Serialize;

use crate::{api::Api, error::Error};

use super::UrlResult;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct OverrideInner {
    name: String,
    satellite: String,
    configuration: String,
    properties: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Override {
    name: String,
    satellite: UrlResult,
    configuration: UrlResult,
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
        let satellite = self.client.path_to_url(format!("satellites/{}", id.into()));

        self.satellite_result(UrlResult::Unchecked(satellite))
    }
}

impl<'a, C> OverrideBuilder<'a, C, NoSatellite> {
    fn satellite_result(self, url: UrlResult) -> OverrideBuilder<'a, C, NoConfig> {
        OverrideBuilder {
            client: self.client,
            state: NoConfig {
                name: self.state.name,
                satellite: url,
            },
        }
    }

    pub fn satellite_url(self, url: impl Into<String>) -> OverrideBuilder<'a, C, NoConfig> {
        self.satellite_result(UrlResult::Checked(url.into()))
    }
}

pub struct NoConfig {
    name: String,
    satellite: UrlResult,
}

impl<'a, C> OverrideBuilder<'a, C, NoConfig>
where
    C: Api,
{
    pub fn satellite_configuration_id(
        self,
        id: impl Into<i32>,
    ) -> OverrideBuilder<'a, C, Override> {
        let url = self.client.path_to_url(format!("satellites/{}", id.into()));
        self.satellite_configuration_result(UrlResult::Unchecked(url))
    }
}

impl<'a, C> OverrideBuilder<'a, C, NoConfig> {
    fn satellite_configuration_result(self, url: UrlResult) -> OverrideBuilder<'a, C, Override> {
        let state = Override {
            name: self.state.name,
            satellite: self.state.satellite,
            configuration: url,
            properties: HashMap::new(),
        };

        OverrideBuilder {
            client: self.client,
            state,
        }
    }

    pub fn satellite_configuration_url(
        self,
        url: impl Into<String>,
    ) -> OverrideBuilder<'a, C, Override> {
        self.satellite_configuration_result(UrlResult::Checked(url.into()))
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

        let url = client.path_to_url("overrides")?;

        let configuration = self.state.configuration.try_convert()?;
        let satellite = self.state.satellite.try_convert()?;

        let dto = OverrideInner {
            name: self.state.name,
            satellite,
            configuration,
            properties: self.state.properties,
        };

        client.post(url, dto).await
    }
}
