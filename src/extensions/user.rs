use crate::api::FreedomApi;
use async_trait::async_trait;
use freedom_models::{account::Account, user::User};

#[async_trait]
pub trait UserExt {
    fn get_id(&self) -> Result<i32, crate::Error>;

    async fn get_account<C>(&self, client: &mut C) -> Result<Account, crate::Error>
    where
        C: FreedomApi + Send;
}

#[async_trait]
impl UserExt for User {
    fn get_id(&self) -> Result<i32, crate::Error> {
        super::get_id("self", &self.links)
    }

    async fn get_account<C>(&self, client: &mut C) -> Result<Account, crate::Error>
    where
        C: FreedomApi + Send,
    {
        super::get_content("account", &self.links, client).await
    }
}
