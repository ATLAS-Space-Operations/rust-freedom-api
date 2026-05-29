use std::future::Future;

use crate::{api::Api, error::Error};
use freedom_models::{
    account::Account,
    user::{User, WhoAmI},
};

pub trait WhoAmIExt {
    fn get_id(&self) -> Result<i32, Error>;

    fn get_user<C>(&self, client: &C) -> impl Future<Output = Result<User, Error>> + Send + Sync
    where
        C: Api;
}

impl WhoAmIExt for WhoAmI {
    fn get_id(&self) -> Result<i32, Error> {
        Ok(self.id)
    }

    async fn get_user<C>(&self, client: &C) -> Result<User, Error>
    where
        C: Api,
    {
        super::get_item("users", &self.links, client).await
    }
}

pub trait UserExt {
    fn get_id(&self) -> Result<i32, Error>;

    fn get_account<C>(
        &self,
        client: &C,
    ) -> impl Future<Output = Result<Account, Error>> + Send + Sync
    where
        C: Api;
}

impl UserExt for User {
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
