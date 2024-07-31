use crate::api::FreedomApi;
use async_trait::async_trait;
use freedom_models::{
    azel::AzEl,
    site::SiteConfiguration,
    task::{Task, TaskRequest},
};

#[async_trait]
pub trait TaskExt {
    fn get_id(&self) -> Result<i32, crate::Error>;

    async fn get_task_request<C>(
        &self,
        client: &C,
    ) -> Result<<C as FreedomApi>::Container<TaskRequest>, crate::Error>
    where
        C: FreedomApi + Send;

    async fn get_config<C>(
        &self,
        client: &C,
    ) -> Result<<C as FreedomApi>::Container<SiteConfiguration>, crate::Error>
    where
        C: FreedomApi + Send;

    async fn get_azel<C>(
        &self,
        client: &C,
    ) -> Result<<C as FreedomApi>::Container<AzEl>, crate::Error>
    where
        C: FreedomApi + Send;
}

#[async_trait]
impl TaskExt for Task {
    fn get_id(&self) -> Result<i32, crate::Error> {
        super::get_id("self", &self.links)
    }

    async fn get_task_request<C>(
        &self,
        client: &C,
    ) -> Result<<C as FreedomApi>::Container<TaskRequest>, crate::Error>
    where
        C: FreedomApi,
    {
        super::get_item("taskRequest", &self.links, client).await
    }

    async fn get_config<C>(
        &self,
        client: &C,
    ) -> Result<<C as FreedomApi>::Container<SiteConfiguration>, crate::Error>
    where
        C: FreedomApi + Send + Sync,
    {
        super::get_item("config", &self.links, client).await
    }

    async fn get_azel<C>(
        &self,
        client: &C,
    ) -> Result<<C as FreedomApi>::Container<AzEl>, crate::Error>
    where
        C: FreedomApi + Send + Sync,
    {
        super::get_item("azel", &self.links, client).await
    }
}
