use crate::error;

use freedom_models::site::{Site, SiteConfiguration};

pub trait SiteConfigurationExt {
    fn get_id(&self) -> Result<i32, error::Error>;
}

impl SiteConfigurationExt for SiteConfiguration {
    fn get_id(&self) -> Result<i32, error::Error> {
        super::get_id("self", &self.links)
    }
}

pub trait SiteExt {
    fn get_id(&self) -> Result<i32, error::Error>;
}

impl SiteExt for Site {
    fn get_id(&self) -> Result<i32, error::Error> {
        super::get_id("self", &self.links)
    }
}
