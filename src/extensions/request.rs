use std::future::Future;

use crate::{api::Api, error::Error};
use freedom_models::{
    band::Band,
    satellite::Satellite,
    site::{Site, SiteConfiguration},
    task::{Task, TaskRequest},
    user::User,
};

pub trait TaskRequestExt {
    fn get_id(&self) -> Result<i32, Error>;

    fn get_task<C>(&self, client: &C) -> impl Future<Output = Result<Task, Error>> + Send + Sync
    where
        C: Api;

    fn get_site<C>(&self, client: &C) -> impl Future<Output = Result<Site, Error>> + Send + Sync
    where
        C: Api;

    fn get_target_bands<C>(
        &self,
        client: &C,
    ) -> impl Future<Output = Result<Vec<Band>, Error>> + Send + Sync
    where
        C: Api;

    fn get_config<C>(
        &self,
        client: &C,
    ) -> impl Future<Output = Result<SiteConfiguration, Error>> + Send + Sync
    where
        C: Api;

    fn get_satellite<C>(
        &self,
        client: &C,
    ) -> impl Future<Output = Result<Satellite, Error>> + Send + Sync
    where
        C: Api;

    fn get_user<C>(&self, client: &C) -> impl Future<Output = Result<User, Error>> + Send + Sync
    where
        C: Api;
}

impl TaskRequestExt for TaskRequest {
    fn get_id(&self) -> Result<i32, Error> {
        super::get_id("self", &self.links)
    }

    async fn get_task<C>(&self, client: &C) -> Result<Task, Error>
    where
        C: Api,
    {
        super::get_item("task", &self.links, client).await
    }

    async fn get_site<C>(&self, client: &C) -> Result<Site, Error>
    where
        C: Api,
    {
        super::get_content("site", &self.links, client).await
    }

    async fn get_target_bands<C>(&self, client: &C) -> Result<Vec<Band>, Error>
    where
        C: Api,
    {
        super::get_embedded("targetBands", &self.links, client).await
    }

    async fn get_config<C>(&self, client: &C) -> Result<SiteConfiguration, Error>
    where
        C: Api,
    {
        tracing::debug!(links = ?self.links, "Getting configuration");
        super::get_content("configuration", &self.links, client).await
    }

    async fn get_satellite<C>(&self, client: &C) -> Result<Satellite, Error>
    where
        C: Api,
    {
        super::get_content("satellite", &self.links, client).await
    }

    async fn get_user<C>(&self, client: &C) -> Result<User, Error>
    where
        C: Api,
    {
        super::get_content("user", &self.links, client).await
    }
}
