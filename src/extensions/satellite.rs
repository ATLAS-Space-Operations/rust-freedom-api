use freedom_models::satellite::Satellite;

use crate::error::Error;

pub trait SatelliteExt {
    fn get_id(&self) -> Result<i32, Error>;
}

impl SatelliteExt for Satellite {
    fn get_id(&self) -> Result<i32, Error> {
        super::get_id("self", &self.links)
    }
}
