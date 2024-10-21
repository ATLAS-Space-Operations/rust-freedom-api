use std::future::Future;

use crate::api::Api;
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
    ) -> impl Future<Output = Result<<C as Api>::Container<TaskRequest>, crate::Error>> + Send
    where
        C: Api + Send;

    fn get_config<C>(
        &self,
        client: &C,
    ) -> impl Future<Output = Result<<C as Api>::Container<SiteConfiguration>, crate::Error>> + Send
    where
        C: Api + Send;

    fn get_azel<C>(
        &self,
        client: &C,
    ) -> impl Future<Output = Result<<C as Api>::Container<AzEl>, crate::Error>> + Send
    where
        C: Api + Send;
}

impl TaskExt for Task {
    fn get_id(&self) -> Result<i32, crate::Error> {
        super::get_id("self", &self.links)
    }

    async fn get_task_request<C>(
        &self,
        client: &C,
    ) -> Result<<C as Api>::Container<TaskRequest>, crate::Error>
    where
        C: Api,
    {
        super::get_item("taskRequest", &self.links, client).await
    }

    async fn get_config<C>(
        &self,
        client: &C,
    ) -> Result<<C as Api>::Container<SiteConfiguration>, crate::Error>
    where
        C: Api + Send + Sync,
    {
        super::get_item("config", &self.links, client).await
    }

    async fn get_azel<C>(&self, client: &C) -> Result<<C as Api>::Container<AzEl>, crate::Error>
    where
        C: Api + Send + Sync,
    {
        super::get_item("azel", &self.links, client).await
    }
}
