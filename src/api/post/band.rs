use freedom_models::{
    band::{BandType, IoConfiguration, IoHardware},
    task::Polarization,
};
use reqwest::Response;
use serde::Serialize;

use crate::{api::Api, error::Error};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BandDetails {
    name: String,
    #[serde(rename(serialize = "type"))]
    typ: BandType,
    frequency_mghz: f64,
    default_band_width_mghz: f64,
    modulation: Option<String>,
    eirp: Option<f64>,
    gain: Option<f64>,
    io_configuration: IoConfiguration,
    polarization: Option<Polarization>,
    manual_transmit_control: bool,
}

pub struct BandDetailsBuilder<'a, C, S> {
    pub(crate) client: &'a C,
    state: S,
}

pub struct NoName;

pub fn new<C>(client: &C) -> BandDetailsBuilder<'_, C, NoName> {
    BandDetailsBuilder {
        client,
        state: NoName,
    }
}

impl<'a, C> BandDetailsBuilder<'a, C, NoName> {
    pub fn name(self, name: impl Into<String>) -> BandDetailsBuilder<'a, C, NoBandType> {
        BandDetailsBuilder {
            client: self.client,
            state: NoBandType { name: name.into() },
        }
    }
}

pub struct NoBandType {
    name: String,
}

impl<'a, C> BandDetailsBuilder<'a, C, NoBandType> {
    pub fn band_type(self, band_type: BandType) -> BandDetailsBuilder<'a, C, NoFrequency> {
        BandDetailsBuilder {
            client: self.client,
            state: NoFrequency {
                name: self.state.name,
                band_type,
            },
        }
    }
}

pub struct NoFrequency {
    name: String,
    band_type: BandType,
}

impl<'a, C> BandDetailsBuilder<'a, C, NoFrequency> {
    pub fn frequency(self, frequency: impl Into<f64>) -> BandDetailsBuilder<'a, C, NoBandWidth> {
        BandDetailsBuilder {
            client: self.client,
            state: NoBandWidth {
                name: self.state.name,
                band_type: self.state.band_type,
                frequency_mghz: frequency.into(),
            },
        }
    }
}

pub struct NoBandWidth {
    name: String,
    band_type: BandType,
    frequency_mghz: f64,
}

impl<'a, C> BandDetailsBuilder<'a, C, NoBandWidth> {
    pub fn default_band_width(
        self,
        bandwidth_mghz: impl Into<f64>,
    ) -> BandDetailsBuilder<'a, C, NoIoConfig> {
        BandDetailsBuilder {
            client: self.client,
            state: NoIoConfig {
                name: self.state.name,
                band_type: self.state.band_type,
                frequency_mghz: self.state.frequency_mghz,
                default_band_width_mghz: bandwidth_mghz.into(),
            },
        }
    }
}

pub struct NoIoConfig {
    name: String,
    band_type: BandType,
    frequency_mghz: f64,
    default_band_width_mghz: f64,
}

impl<'a, C> BandDetailsBuilder<'a, C, NoIoConfig> {
    pub fn io_hardware(self, hardware: IoHardware) -> BandDetailsBuilder<'a, C, BandDetails> {
        let state = BandDetails {
            name: self.state.name,
            typ: self.state.band_type,
            frequency_mghz: self.state.frequency_mghz,
            default_band_width_mghz: self.state.default_band_width_mghz,
            io_configuration: IoConfiguration {
                start_hex_pattern: None,
                end_hex_pattern: None,
                strip_pattern: false,
                io_hardware: Some(hardware),
            },
            modulation: None,
            eirp: None,
            gain: None,
            polarization: None,
            manual_transmit_control: false,
        };

        BandDetailsBuilder {
            client: self.client,
            state,
        }
    }
}

impl<C> BandDetailsBuilder<'_, C, BandDetails> {
    pub fn polarization(mut self, polarization: Polarization) -> Self {
        self.state.polarization = Some(polarization);
        self
    }

    pub fn modulation(mut self, modulation: impl Into<String>) -> Self {
        self.state.modulation = Some(modulation.into());
        self
    }

    pub fn effective_isotropic_radiative_power(mut self, eirp: impl Into<f64>) -> Self {
        self.state.eirp = Some(eirp.into());
        self
    }

    pub fn gain(mut self, gain: impl Into<f64>) -> Self {
        self.state.gain = Some(gain.into());
        self
    }

    pub fn manual_transmit_control(mut self, control: bool) -> Self {
        self.state.manual_transmit_control = control;
        self
    }
}

impl<C> BandDetailsBuilder<'_, C, BandDetails>
where
    C: Api,
{
    pub async fn send(self) -> Result<Response, Error> {
        let client = self.client;

        let url = client.path_to_url("satellite_bands");
        client.post(url, self.state).await
    }
}
