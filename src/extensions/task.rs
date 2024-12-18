use std::future::Future;

use crate::{api::Api, error::Error};
use freedom_models::{
    azel::AzEl,
    site::SiteConfiguration,
    task::{Task, TaskRequest},
};

pub trait TaskExt {
    fn get_id(&self) -> Result<i32, Error>;

    fn get_task_request<C>(
        &self,
        client: &C,
    ) -> impl Future<Output = Result<<C as Api>::Container<TaskRequest>, Error>> + Send
    where
        C: Api + Send;

    fn get_config<C>(
        &self,
        client: &C,
    ) -> impl Future<Output = Result<<C as Api>::Container<SiteConfiguration>, Error>> + Send
    where
        C: Api + Send;

    fn get_azel<C>(
        &self,
        client: &C,
    ) -> impl Future<Output = Result<<C as Api>::Container<AzEl>, Error>> + Send
    where
        C: Api + Send;
}

impl TaskExt for Task {
    fn get_id(&self) -> Result<i32, Error> {
        super::get_id("self", &self.links)
    }

    async fn get_task_request<C>(
        &self,
        client: &C,
    ) -> Result<<C as Api>::Container<TaskRequest>, Error>
    where
        C: Api,
    {
        super::get_item("taskRequest", &self.links, client).await
    }

    async fn get_config<C>(
        &self,
        client: &C,
    ) -> Result<<C as Api>::Container<SiteConfiguration>, Error>
    where
        C: Api + Send + Sync,
    {
        super::get_item("config", &self.links, client).await
    }

    async fn get_azel<C>(&self, client: &C) -> Result<<C as Api>::Container<AzEl>, Error>
    where
        C: Api + Send + Sync,
    {
        super::get_item("azel", &self.links, client).await
    }
}
