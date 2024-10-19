use std::future::Future;

use crate::api::FreedomApi;
use freedom_models::{
    azel::AzEl,
    site::SiteConfiguration,
    task::{Task, TaskRequest},
};

pub trait TaskExt {
    fn get_id(&self) -> Result<i32, crate::Error>;

    fn get_task_request<C>(
        &self,
        client: &C,
    ) -> impl Future<Output = Result<<C as FreedomApi>::Container<TaskRequest>, crate::Error>> + Send
    where
        C: FreedomApi + Send;

    fn get_config<C>(
        &self,
        client: &C,
    ) -> impl Future<Output = Result<<C as FreedomApi>::Container<SiteConfiguration>, crate::Error>> + Send
    where
        C: FreedomApi + Send;

    fn get_azel<C>(
        &self,
        client: &C,
    ) -> impl Future<Output = Result<<C as FreedomApi>::Container<AzEl>, crate::Error>> + Send
    where
        C: FreedomApi + Send;
}

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
