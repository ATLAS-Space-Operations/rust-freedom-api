use std::future::Future;

use crate::{api::Api, error::Error};
use freedom_models::{account::Account, satellite::Satellite, user::User};

pub trait AccountExt {
    fn get_id(&self) -> Result<i32, Error>;

    fn get_users<C>(
        &self,
        client: &C,
    ) -> impl Future<Output = Result<Vec<User>, Error>> + Send + Sync
    where
        C: Api;

    fn get_satellites<C>(
        &self,
        client: &C,
    ) -> impl Future<Output = Result<Vec<Satellite>, Error>> + Send + Sync
    where
        C: Api;
}

impl AccountExt for Account {
    fn get_id(&self) -> Result<i32, Error> {
        super::get_id("self", &self.links)
    }

    async fn get_users<C>(&self, client: &C) -> Result<Vec<User>, Error>
    where
        C: Api,
    {
        super::get_item("users", &self.links, client).await
    }

    async fn get_satellites<C>(&self, client: &C) -> Result<Vec<Satellite>, Error>
    where
        C: Api,
    {
        super::get_embedded("satellites", &self.links, client).await
    }
}
