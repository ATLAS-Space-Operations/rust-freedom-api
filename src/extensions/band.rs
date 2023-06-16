use freedom_models::band::Band;

pub trait BandExt {
    fn get_id(&self) -> Result<i32, crate::Error>;
}

impl BandExt for Band {
    fn get_id(&self) -> Result<i32, crate::Error> {
        super::get_id("self", &self.links)
    }
}
