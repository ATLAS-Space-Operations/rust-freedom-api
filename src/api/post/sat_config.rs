use reqwest::Response;
use serde::Serialize;

use crate::{api::FreedomApi, Error};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SatelliteConfiguration {
    name: String,
    doppler: Option<bool>,
    notes: Option<String>,
    band_details: Vec<String>,
}

pub struct NoName;

pub struct SatelliteConfigurationBuilder<'a, C, S> {
    pub(crate) client: &'a C,
    state: S,
}

pub(crate) fn new<C>(client: &C) -> SatelliteConfigurationBuilder<'_, C, NoName> {
    SatelliteConfigurationBuilder {
        client,
        state: NoName,
    }
}

impl<'a, C> SatelliteConfigurationBuilder<'a, C, NoName> {
    pub fn name(self, name: impl Into<String>) -> SatelliteConfigurationBuilder<'a, C, NoBand> {
        SatelliteConfigurationBuilder {
            client: self.client,
            state: NoBand { name: name.into() },
        }
    }
}

pub struct NoBand {
    name: String,
}

impl<'a, C> SatelliteConfigurationBuilder<'a, C, NoBand> {
    pub fn band_urls(
        self,
        urls: impl IntoIterator<Item = String>,
    ) -> SatelliteConfigurationBuilder<'a, C, SatelliteConfiguration> {
        let band_details: Vec<_> = urls.into_iter().collect();

        let state = SatelliteConfiguration {
            name: self.state.name,
            doppler: None,
            notes: None,
            band_details,
        };

        SatelliteConfigurationBuilder {
            client: self.client,
            state,
        }
    }
}

impl<'a, C> SatelliteConfigurationBuilder<'a, C, NoBand>
where
    C: FreedomApi,
{
    pub fn band_ids(
        self,
        ids: impl IntoIterator<Item = i32>,
    ) -> SatelliteConfigurationBuilder<'a, C, SatelliteConfiguration> {
        let client = self.client;
        let bands = ids.into_iter().map(|id| {
            client
                .path_to_url(format!("satellite_bands/{}", id))
                .to_string()
        });

        self.band_urls(bands)
    }
}

impl<'a, C> SatelliteConfigurationBuilder<'a, C, SatelliteConfiguration> {
    pub fn doppler(mut self, doppler: bool) -> Self {
        self.state.doppler = Some(doppler);
        self
    }

    pub fn notes(mut self, notes: impl Into<String>) -> Self {
        self.state.notes = Some(notes.into());
        self
    }
}

impl<'a, C> SatelliteConfigurationBuilder<'a, C, SatelliteConfiguration>
where
    C: FreedomApi,
{
    pub async fn send(self) -> Result<Response, Error> {
        let client = self.client;

        let url = client.path_to_url("satellite_configurations");
        client.post(url, self.state).await
    }
}
