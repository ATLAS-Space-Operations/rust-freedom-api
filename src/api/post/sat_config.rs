use reqwest::Response;
use serde::Serialize;

use crate::{api::Api, error::Error};

use super::UrlResult;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct SatelliteConfigurationInner {
    name: String,
    doppler: Option<bool>,
    notes: Option<String>,
    band_details: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SatelliteConfiguration {
    name: String,
    doppler: Option<bool>,
    notes: Option<String>,
    band_details: Vec<UrlResult>,
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
        let band_details: Vec<_> = urls.into_iter().map(UrlResult::Checked).collect();
        self.band_results(band_details)
    }

    fn band_results(
        self,
        ids: impl IntoIterator<Item = UrlResult>,
    ) -> SatelliteConfigurationBuilder<'a, C, SatelliteConfiguration> {
        let state = SatelliteConfiguration {
            name: self.state.name,
            doppler: None,
            notes: None,
            band_details: ids.into_iter().collect(),
        };

        SatelliteConfigurationBuilder {
            client: self.client,
            state,
        }
    }
}

impl<'a, C> SatelliteConfigurationBuilder<'a, C, NoBand>
where
    C: Api,
{
    pub fn band_ids(
        self,
        ids: impl IntoIterator<Item = i32>,
    ) -> SatelliteConfigurationBuilder<'a, C, SatelliteConfiguration> {
        let client = self.client;
        let results = ids
            .into_iter()
            .map(|id| UrlResult::Unchecked(client.path_to_url(format!("satellite_bands/{}", id))))
            .collect::<Vec<_>>();
        self.band_results(results)
    }
}

impl<C> SatelliteConfigurationBuilder<'_, C, SatelliteConfiguration> {
    pub fn doppler(mut self, doppler: bool) -> Self {
        self.state.doppler = Some(doppler);
        self
    }

    pub fn notes(mut self, notes: impl Into<String>) -> Self {
        self.state.notes = Some(notes.into());
        self
    }
}

impl<C> SatelliteConfigurationBuilder<'_, C, SatelliteConfiguration>
where
    C: Api,
{
    pub async fn send(self) -> Result<Response, Error> {
        let client = self.client;

        let mut band_details = Vec::new();
        for result in self.state.band_details {
            let band = result.try_convert()?;
            band_details.push(band);
        }

        let dto = SatelliteConfigurationInner {
            name: self.state.name,
            doppler: self.state.doppler,
            notes: self.state.notes,
            band_details,
        };

        let url = client.path_to_url("satellite_configurations")?;
        client.post(url, dto).await
    }
}
