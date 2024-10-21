use std::future::Future;

use crate::api::Api;
use freedom_models::{account::Account, user::User};

pub trait UserExt {
    fn get_id(&self) -> Result<i32, crate::Error>;

    fn get_account<C>(
        &self,
        client: &C,
    ) -> impl Future<Output = Result<Account, crate::Error>> + Send
    where
        C: Api + Send;
}

impl UserExt for User {
    fn get_id(&self) -> Result<i32, crate::Error> {
        super::get_id("self", &self.links)
    }

    async fn get_account<C>(&self, client: &C) -> Result<Account, crate::Error>
    where
        C: Api + Send,
    {
        super::get_content("account", &self.links, client).await
    }
}
