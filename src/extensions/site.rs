use crate::{Api, error::Error};

use freedom_models::site::{Site, SiteConfiguration};

pub trait SiteConfigurationExt {
    fn get_id(&self) -> Result<i32, Error>;

    fn get_site<C>(&self, client: &C) -> impl Future<Output = Result<Site, Error>> + Send + Sync
    where
        C: Api;
}

impl SiteConfigurationExt for SiteConfiguration {
    fn get_id(&self) -> Result<i32, Error> {
        super::get_id("self", &self.links)
    }

    async fn get_site<C>(&self, client: &C) -> Result<Site, Error>
    where
        C: Api,
    {
        super::get_content("site", &self.links, client).await
    }
}

pub trait SiteExt {
    fn get_id(&self) -> Result<i32, Error>;

    fn get_site_configurations<C>(
        &self,
        client: &C,
    ) -> impl Future<Output = Result<Vec<SiteConfiguration>, Error>> + Send + Sync
    where
        C: Api;
}

impl SiteExt for Site {
    fn get_id(&self) -> Result<i32, Error> {
        super::get_id("self", &self.links)
    }

    async fn get_site_configurations<C>(&self, client: &C) -> Result<Vec<SiteConfiguration>, Error>
    where
        C: Api,
    {
        super::get_embedded("configurations", &self.links, client).await
    }
}
