use std::collections::HashMap;

use derive_builder::Builder;
use freedom_models::{
    band::{BandType, IoConfiguration},
    task::{Polarization, TaskType},
};
use reqwest::Response;
use serde::Serialize;
use time::OffsetDateTime;

use crate::{api::FreedomApi, prelude::BuilderError, Error};

#[derive(Builder, Debug, Clone, PartialEq, Serialize)]
#[builder(setter(into), build_fn(skip), pattern = "owned")]
#[serde(rename_all = "camelCase")]
pub struct BandDetails<'a, C> {
    #[serde(skip_serializing)]
    #[builder(setter(custom))]
    client: &'a C,
    name: String,
    #[serde(rename(serialize = "type"))]
    #[builder(setter(name = "band_type"))]
    typ: BandType,
    frequency_mghz: f64,
    default_band_width_mghz: f64,
    modulation: String,
    eirp: f64,
    gain: f64,
    io_configuration: IoConfiguration,
    polarization: Polarization,
    manual_transmit_control: bool,
}

impl<'a, C> BandDetailsBuilder<'a, C>
where
    C: FreedomApi,
{
    pub(crate) fn client(client: &'a C) -> Self {
        Self {
            client: Some(client),
            ..Default::default()
        }
    }

    pub async fn send(self) -> Result<Response, Error> {
        let client = self.client.unwrap();

        let details = BandDetails {
            client,
            name: self.name.unwrap_or_default(),
            typ: self.typ.unwrap_or(BandType::Receive),
            frequency_mghz: self.frequency_mghz.unwrap_or_default(),
            default_band_width_mghz: self.default_band_width_mghz.unwrap_or(1.0),
            modulation: self.modulation.unwrap_or_default(),
            eirp: self.eirp.unwrap_or_default(),
            gain: self.gain.unwrap_or_default(),
            io_configuration: self.io_configuration.unwrap_or(IoConfiguration {
                start_hex_pattern: None,
                end_hex_pattern: None,
                strip_pattern: false,
                io_hardware: None,
            }),
            polarization: self.polarization.unwrap_or_default(),
            manual_transmit_control: self.manual_transmit_control.unwrap_or_default(),
        };

        let url = client.path_to_url("satellite_bands");
        client.post(url, details).await
    }
}

#[derive(Builder, Debug, Clone, PartialEq, Serialize)]
#[builder(setter(into), build_fn(skip), pattern = "owned")]
#[serde(rename_all = "camelCase")]
pub struct SatelliteConfiguration<'a, C> {
    #[serde(skip_serializing)]
    #[builder(setter(custom))]
    client: &'a C,
    name: String,
    doppler: bool,
    notes: String,
    #[builder(setter(custom), default)]
    band_details: Vec<String>,
}

impl<'a, C> SatelliteConfigurationBuilder<'a, C>
where
    C: FreedomApi,
{
    pub(crate) fn client(client: &'a C) -> Self {
        Self {
            client: Some(client),
            ..Default::default()
        }
    }

    pub fn band_details(mut self, band_details: impl IntoIterator<Item = u32>) -> Self {
        let client = self.client.unwrap();
        let band_details: Vec<_> = band_details
            .into_iter()
            .map(|id| {
                client
                    .path_to_url(format!("satellite_bands/{}", id))
                    .to_string()
            })
            .collect();
        self.band_details = Some(band_details);
        self
    }

    pub async fn send(self) -> Result<Response, Error> {
        let client = self.client.unwrap();

        let details = SatelliteConfiguration {
            client,
            name: self.name.unwrap_or_default(),
            doppler: self.doppler.unwrap_or_default(),
            notes: self.notes.unwrap_or_default(),
            band_details: self.band_details.unwrap_or_default(),
        };

        let url = client.path_to_url("satellite_configurations");
        client.post(url, details).await
    }
}

#[derive(Builder, Debug, Clone, PartialEq, Serialize)]
#[builder(setter(into), build_fn(skip), pattern = "owned")]
#[serde(rename_all = "camelCase")]
pub struct Satellite<'a, C> {
    #[serde(skip_serializing)]
    #[builder(setter(custom))]
    client: &'a C,
    name: String,
    description: String,
    norad_cat_id: i32,
    #[builder(setter(custom), default)]
    configuration: String,
}

impl<'a, C> SatelliteBuilder<'a, C>
where
    C: FreedomApi,
{
    pub(crate) fn client(client: &'a C) -> Self {
        Self {
            client: Some(client),
            ..Default::default()
        }
    }

    pub fn configuration(mut self, satellite_configuration_id: u32) -> Self {
        let client = self.client.unwrap();
        let configuration = client
            .path_to_url(format!(
                "satellite_configurations/{}",
                satellite_configuration_id
            ))
            .to_string();
        self.configuration = Some(configuration);
        self
    }

    pub async fn send(self) -> Result<Response, Error> {
        let client = self.client.unwrap();

        let details = Satellite {
            client,
            name: self.name.unwrap_or_default(),
            description: self.description.unwrap_or_default(),
            norad_cat_id: self.norad_cat_id.unwrap_or_default(),
            configuration: self.configuration.unwrap_or_default(),
        };

        let url = client.path_to_url("satellites");
        client.post(url, details).await
    }
}

#[derive(Builder, Debug, Clone, PartialEq, Serialize)]
#[builder(setter(into), build_fn(skip), pattern = "owned")]
#[serde(rename_all = "camelCase")]
pub struct Override<'a, C> {
    #[serde(skip_serializing)]
    #[builder(setter(custom))]
    client: &'a C,
    #[builder(setter(custom))]
    name: Option<String>,
    #[builder(setter(custom), default)]
    satellite: String,
    #[builder(setter(custom), default)]
    configuration: String,
    #[builder(setter(custom), field(ty = "HashMap<String, String>"))]
    properties: HashMap<String, String>,
}

impl<'a, C> OverrideBuilder<'a, C>
where
    C: FreedomApi,
{
    pub(crate) fn client(client: &'a C) -> Self {
        Self {
            client: Some(client),
            ..Default::default()
        }
    }

    pub fn add_property(mut self, key: impl Into<String>, value: impl std::fmt::Display) -> Self {
        let _ = self.properties.insert(key.into(), value.to_string());
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(Some(name.into()));
        self
    }

    pub fn satellite(mut self, satellite_id: u32) -> Self {
        let client = self.client.unwrap();
        let satellite = client
            .path_to_url(format!("satellites/{}", satellite_id))
            .to_string();
        self.satellite = Some(satellite);
        self
    }

    pub fn configuration(mut self, satellite_configuration_id: u32) -> Self {
        let client = self.client.unwrap();
        let configuration = client
            .path_to_url(format!(
                "satellite_configurations/{}",
                satellite_configuration_id
            ))
            .to_string();
        self.configuration = Some(configuration);
        self
    }

    pub async fn send(self) -> Result<Response, Error> {
        let client = self.client.unwrap();

        let override_builder = Override {
            client,
            name: self.name.unwrap_or_default(),
            configuration: self.configuration.unwrap_or_default(),
            satellite: self.satellite.unwrap_or_default(),
            properties: self.properties,
        };

        let url = client.path_to_url("overrides");
        client.post(url, override_builder).await
    }
}

#[derive(Builder, Debug, Clone, PartialEq, Serialize)]
#[builder(setter(into), build_fn(skip), pattern = "owned")]
#[serde(rename_all = "camelCase")]
pub struct User<'a, C> {
    #[serde(skip_serializing)]
    #[builder(setter(custom))]
    client: &'a C,
    #[serde(skip_serializing)]
    account_id: i32,
    first_name: String,
    last_name: String,
    email: String,
    machine_service: bool,
    #[builder(setter(custom), field(ty = "Vec<String>"))]
    roles: Vec<String>,
}

impl<'a, C> UserBuilder<'a, C>
where
    C: FreedomApi,
{
    pub(crate) fn client(client: &'a C) -> Self {
        Self {
            client: Some(client),
            ..Default::default()
        }
    }

    pub async fn send(self) -> Result<Response, Error> {
        let client = self.client.unwrap();

        let account_id = self.account_id.ok_or(BuilderError::AccountId)?;
        let user = User {
            client,
            account_id,
            first_name: self.first_name.unwrap_or_default(),
            last_name: self.last_name.unwrap_or_default(),
            email: self.email.unwrap_or_default(),
            machine_service: self.machine_service.unwrap_or_default(),
            roles: self.roles,
        };

        let url = client.path_to_url(format!("accounts/{}/newuser", user.account_id));
        client.post(url, user).await
    }
}

#[derive(Builder, Debug, Clone, PartialEq, Serialize)]
#[builder(build_fn(skip), pattern = "owned")]
#[serde(rename_all = "camelCase")]
pub struct TaskRequest<'a, C> {
    #[serde(skip_serializing)]
    #[builder(setter(custom))]
    client: &'a C,
    #[serde(rename(serialize = "type"))]
    #[builder(setter(name = "task_type"))]
    typ: TaskType,
    #[builder(setter(custom))]
    site: String,
    #[builder(setter(custom))]
    satellite: String,
    #[builder(setter(custom))]
    configuration: String,
    #[builder(setter(custom), field(ty = "Vec<String>"))]
    target_bands: Vec<String>,
    #[builder(
        setter(name = "target_date_utc", strip_option),
        field(ty = "Option<OffsetDateTime>")
    )]
    target_date: String,
    duration: u64,
    #[builder(field(ty = "Option<u64>"), setter(strip_option))]
    minimum_duration: Option<u64>,
    #[builder(field(ty = "Option<u64>"), setter(strip_option))]
    hours_of_flex: Option<u64>,
    #[builder(field(ty = "Option<String>"), setter(strip_option))]
    test_file: Option<String>,
    #[serde(rename(serialize = "override"))]
    #[builder(field(ty = "Option<String>"))]
    with_override: Option<String>,
}

impl<'a, C> TaskRequestBuilder<'a, C>
where
    C: FreedomApi,
{
    pub(crate) fn client(client: &'a C) -> Self {
        Self {
            client: Some(client),
            ..Default::default()
        }
    }

    pub fn site(mut self, site_id: u32) -> Self {
        let client = self.client.unwrap();
        let site = client.path_to_url(format!("sites/{}", site_id)).to_string();
        self.site = Some(site);
        self
    }

    pub fn target_bands(mut self, band_ids: impl IntoIterator<Item = i32>) -> Self {
        let client = self.client.unwrap();
        let target_bands: Vec<_> = band_ids
            .into_iter()
            .map(|id| {
                client
                    .path_to_url(format!("satellite_bands/{}", id))
                    .to_string()
            })
            .collect();
        self.target_bands = target_bands;
        self
    }

    pub fn satellite(mut self, satellite_id: u32) -> Self {
        let client = self.client.unwrap();
        let satellite = client
            .path_to_url(format!("satellites/{}", satellite_id))
            .to_string();
        self.satellite = Some(satellite);
        self
    }

    pub fn configuration(mut self, configuration_id: u32) -> Self {
        let client = self.client.unwrap();
        let configuration = client
            .path_to_url(format!("configurations/{}", configuration_id))
            .to_string();
        self.configuration = Some(configuration);
        self
    }

    pub async fn send(self) -> Result<Response, Error> {
        use time::macros::format_description;
        let item = format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]Z");

        let client = self.client.unwrap();

        let target_date = self.target_date.ok_or(BuilderError::TargetDate)?;
        let target_date = target_date.format(item)?;
        let task_type = self.typ.ok_or(BuilderError::TaskType)?;
        let request = TaskRequest {
            client,
            typ: task_type,
            site: self.site.ok_or(BuilderError::SiteId)?,
            satellite: self.satellite.ok_or(BuilderError::SatelliteId)?,
            configuration: self.configuration.ok_or(BuilderError::ConfigurationId)?,
            target_date,
            minimum_duration: self.minimum_duration,
            duration: self.duration.ok_or(BuilderError::Duration)?,
            hours_of_flex: self.hours_of_flex,
            test_file: self.test_file,
            with_override: self.with_override,
            target_bands: self.target_bands,
        };

        println!("{}", serde_json::to_string_pretty(&request).unwrap());

        let url = client.path_to_url("requests");
        client.post(url, request).await
    }
}
