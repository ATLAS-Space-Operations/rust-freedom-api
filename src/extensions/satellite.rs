use freedom_models::satellite::Satellite;

pub trait SatelliteExt {
    fn get_id(&self) -> Result<i32, crate::Error>;
}

impl SatelliteExt for Satellite {
    fn get_id(&self) -> Result<i32, crate::Error> {
        super::get_id("self", &self.links)
    }
}
