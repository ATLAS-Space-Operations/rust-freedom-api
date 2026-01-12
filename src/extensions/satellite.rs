use freedom_models::{
    account::Account, satellite::Satellite, satellite_configuration::SatelliteConfiguration,
};

use crate::{Api, error::Error};

pub trait SatelliteExt {
    fn get_id(&self) -> Result<i32, Error>;

    fn get_account<C>(
        &self,
        client: &C,
    ) -> impl Future<Output = Result<Account, Error>> + Send + Sync
    where
        C: Api;

    fn get_satellite_configuration<C>(
        &self,
        client: &C,
    ) -> impl Future<Output = Result<SatelliteConfiguration, Error>> + Send + Sync
    where
        C: Api;
}

impl SatelliteExt for Satellite {
    fn get_id(&self) -> Result<i32, Error> {
        super::get_id("self", &self.links)
    }

    async fn get_account<C>(&self, client: &C) -> Result<Account, Error>
    where
        C: Api,
    {
        super::get_item("account", &self.links, client).await
    }

    async fn get_satellite_configuration<C>(
        &self,
        client: &C,
    ) -> Result<SatelliteConfiguration, Error>
    where
        C: Api,
    {
        super::get_item("configuration", &self.links, client).await
    }
}
