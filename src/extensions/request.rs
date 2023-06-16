use crate::api::FreedomApi;
use async_trait::async_trait;
use freedom_models::{
    band::Band,
    satellite::Satellite,
    site::{Site, SiteConfiguration},
    task::{Task, TaskRequest},
    user::User,
};

#[async_trait]
pub trait TaskRequestExt {
    fn get_id(&self) -> Result<i32, crate::Error>;

    async fn get_task<C>(
        &self,
        client: &mut C,
    ) -> Result<<C as FreedomApi>::Container<Task>, crate::Error>
    where
        C: FreedomApi + Send;

    async fn get_site<C>(&self, client: &mut C) -> Result<Site, crate::Error>
    where
        C: FreedomApi + Send;

    async fn get_target_bands<C>(
        &self,
        client: &mut C,
    ) -> Result<<C as FreedomApi>::Container<Vec<Band>>, crate::Error>
    where
        C: FreedomApi + Send;

    async fn get_config<C>(&self, client: &mut C) -> Result<SiteConfiguration, crate::Error>
    where
        C: FreedomApi + Send;

    async fn get_satellite<C>(&self, client: &mut C) -> Result<Satellite, crate::Error>
    where
        C: FreedomApi + Send;

    async fn get_user<C>(&self, client: &mut C) -> Result<User, crate::Error>
    where
        C: FreedomApi + Send;
}

#[async_trait]
impl TaskRequestExt for TaskRequest {
    fn get_id(&self) -> Result<i32, crate::Error> {
        super::get_id("self", &self.links)
    }

    async fn get_task<C>(
        &self,
        client: &mut C,
    ) -> Result<<C as FreedomApi>::Container<Task>, crate::Error>
    where
        C: FreedomApi + Send,
    {
        super::get_item("task", &self.links, client).await
    }

    async fn get_site<C>(&self, client: &mut C) -> Result<Site, crate::Error>
    where
        C: FreedomApi + Send,
    {
        super::get_content("site", &self.links, client).await
    }

    async fn get_target_bands<C>(
        &self,
        client: &mut C,
    ) -> Result<<C as FreedomApi>::Container<Vec<Band>>, crate::Error>
    where
        C: FreedomApi + Send,
    {
        super::get_embedded("targetBands", &self.links, client).await
    }

    async fn get_config<C>(&self, client: &mut C) -> Result<SiteConfiguration, crate::Error>
    where
        C: FreedomApi + Send,
    {
        tracing::debug!(links = ?self.links, "Getting configuration");
        super::get_content("configuration", &self.links, client).await
    }

    async fn get_satellite<C>(&self, client: &mut C) -> Result<Satellite, crate::Error>
    where
        C: FreedomApi + Send,
    {
        super::get_content("satellite", &self.links, client).await
    }

    async fn get_user<C>(&self, client: &mut C) -> Result<User, crate::Error>
    where
        C: FreedomApi + Send,
    {
        super::get_content("user", &self.links, client).await
    }
}
