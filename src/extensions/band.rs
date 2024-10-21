use crate::error::Error;
use freedom_models::band::Band;

pub trait BandExt {
    fn get_id(&self) -> Result<i32, Error>;
}

impl BandExt for Band {
    fn get_id(&self) -> Result<i32, Error> {
        super::get_id("self", &self.links)
    }
}
