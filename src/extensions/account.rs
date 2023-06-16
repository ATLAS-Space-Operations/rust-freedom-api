use crate::api::FreedomApi;
use async_trait::async_trait;
use freedom_models::{account::Account, satellite::Satellite, user::User};

#[async_trait]
pub trait AccountExt {
    fn get_id(&self) -> Result<i32, crate::Error>;

    async fn get_users<C>(
        &self,
        client: &mut C,
    ) -> Result<<C as FreedomApi>::Container<Vec<User>>, crate::Error>
    where
        C: FreedomApi + Send;

    async fn get_satellites<C>(
        &self,
        client: &mut C,
    ) -> Result<<C as FreedomApi>::Container<Vec<Satellite>>, crate::Error>
    where
        C: FreedomApi + Send;
}

#[async_trait]
impl AccountExt for Account {
    fn get_id(&self) -> Result<i32, crate::Error> {
        super::get_id("self", &self.links)
    }

    async fn get_users<C>(
        &self,
        client: &mut C,
    ) -> Result<<C as FreedomApi>::Container<Vec<User>>, crate::Error>
    where
        C: FreedomApi + Send,
    {
        super::get_item("users", &self.links, client).await
    }

    async fn get_satellites<C>(
        &self,
        client: &mut C,
    ) -> Result<<C as FreedomApi>::Container<Vec<Satellite>>, crate::Error>
    where
        C: FreedomApi + Send,
    {
        super::get_embedded("satellites", &self.links, client).await
    }
}
