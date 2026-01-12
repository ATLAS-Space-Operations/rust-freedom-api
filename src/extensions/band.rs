use crate::{Api, error::Error};
use freedom_models::{account::Account, band::Band};

pub trait BandExt {
    fn get_id(&self) -> Result<i32, Error>;

    fn get_account<C>(
        &self,
        client: &C,
    ) -> impl Future<Output = Result<Account, Error>> + Send + Sync
    where
        C: Api;
}

impl BandExt for Band {
    fn get_id(&self) -> Result<i32, Error> {
        super::get_id("self", &self.links)
    }

    async fn get_account<C>(&self, client: &C) -> Result<Account, Error>
    where
        C: Api,
    {
        super::get_content("account", &self.links, client).await
    }
}
