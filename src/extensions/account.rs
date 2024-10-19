use std::future::Future;

use crate::api::FreedomApi;
use freedom_models::{account::Account, satellite::Satellite, user::User};

pub trait AccountExt {
    fn get_id(&self) -> Result<i32, crate::Error>;

    fn get_users<C>(
        &self,
        client: &C,
    ) -> impl Future<Output = Result<<C as FreedomApi>::Container<Vec<User>>, crate::Error>> + Send
    where
        C: FreedomApi + Send;

    fn get_satellites<C>(
        &self,
        client: &C,
    ) -> impl Future<Output = Result<<C as FreedomApi>::Container<Vec<Satellite>>, crate::Error>>
    where
        C: FreedomApi + Send;
}

impl AccountExt for Account {
    fn get_id(&self) -> Result<i32, crate::Error> {
        super::get_id("self", &self.links)
    }

    async fn get_users<C>(
        &self,
        client: &C,
    ) -> Result<<C as FreedomApi>::Container<Vec<User>>, crate::Error>
    where
        C: FreedomApi + Send,
    {
        super::get_item("users", &self.links, client).await
    }

    async fn get_satellites<C>(
        &self,
        client: &C,
    ) -> Result<<C as FreedomApi>::Container<Vec<Satellite>>, crate::Error>
    where
        C: FreedomApi + Send,
    {
        super::get_embedded("satellites", &self.links, client).await
    }
}
