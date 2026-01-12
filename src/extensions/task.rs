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
    ) -> impl Future<Output = Result<TaskRequest, Error>> + Send + Sync
    where
        C: Api;

    fn get_site_configuration<C>(
        &self,
        client: &C,
    ) -> impl Future<Output = Result<SiteConfiguration, Error>> + Send + Sync
    where
        C: Api;

    fn get_azel<C>(&self, client: &C) -> impl Future<Output = Result<AzEl, Error>> + Send + Sync
    where
        C: Api;
}

impl TaskExt for Task {
    fn get_id(&self) -> Result<i32, Error> {
        super::get_id("self", &self.links)
    }

    async fn get_task_request<C>(&self, client: &C) -> Result<TaskRequest, Error>
    where
        C: Api,
    {
        super::get_item("taskRequest", &self.links, client).await
    }

    async fn get_site_configuration<C>(&self, client: &C) -> Result<SiteConfiguration, Error>
    where
        C: Api,
    {
        super::get_item("config", &self.links, client).await
    }

    async fn get_azel<C>(&self, client: &C) -> Result<AzEl, Error>
    where
        C: Api,
    {
        super::get_item("azel", &self.links, client).await
    }
}
