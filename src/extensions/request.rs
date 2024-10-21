use std::future::Future;

use crate::api::Api;
use freedom_models::{
    band::Band,
    satellite::Satellite,
    site::{Site, SiteConfiguration},
    task::{Task, TaskRequest},
    user::User,
};

pub trait TaskRequestExt {
    fn get_id(&self) -> Result<i32, crate::Error>;

    fn get_task<C>(
        &self,
        client: &C,
    ) -> impl Future<Output = Result<<C as Api>::Container<Task>, crate::Error>> + Send
    where
        C: Api + Send;

    fn get_site<C>(&self, client: &C) -> impl Future<Output = Result<Site, crate::Error>> + Send
    where
        C: Api + Send;

    fn get_target_bands<C>(
        &self,
        client: &C,
    ) -> impl Future<Output = Result<<C as Api>::Container<Vec<Band>>, crate::Error>> + Send
    where
        C: Api + Send;

    fn get_config<C>(
        &self,
        client: &C,
    ) -> impl Future<Output = Result<SiteConfiguration, crate::Error>> + Send
    where
        C: Api + Send;

    fn get_satellite<C>(
        &self,
        client: &C,
    ) -> impl Future<Output = Result<Satellite, crate::Error>> + Send
    where
        C: Api + Send;

    fn get_user<C>(&self, client: &C) -> impl Future<Output = Result<User, crate::Error>> + Send
    where
        C: Api + Send;
}

impl TaskRequestExt for TaskRequest {
    fn get_id(&self) -> Result<i32, crate::Error> {
        super::get_id("self", &self.links)
    }

    async fn get_task<C>(&self, client: &C) -> Result<<C as Api>::Container<Task>, crate::Error>
    where
        C: Api + Send,
    {
        super::get_item("task", &self.links, client).await
    }

    async fn get_site<C>(&self, client: &C) -> Result<Site, crate::Error>
    where
        C: Api + Send,
    {
        super::get_content("site", &self.links, client).await
    }

    async fn get_target_bands<C>(
        &self,
        client: &C,
    ) -> Result<<C as Api>::Container<Vec<Band>>, crate::Error>
    where
        C: Api + Send,
    {
        super::get_embedded("targetBands", &self.links, client).await
    }

    async fn get_config<C>(&self, client: &C) -> Result<SiteConfiguration, crate::Error>
    where
        C: Api + Send,
    {
        tracing::debug!(links = ?self.links, "Getting configuration");
        super::get_content("configuration", &self.links, client).await
    }

    async fn get_satellite<C>(&self, client: &C) -> Result<Satellite, crate::Error>
    where
        C: Api + Send,
    {
        super::get_content("satellite", &self.links, client).await
    }

    async fn get_user<C>(&self, client: &C) -> Result<User, crate::Error>
    where
        C: Api + Send,
    {
        super::get_content("user", &self.links, client).await
    }
}
