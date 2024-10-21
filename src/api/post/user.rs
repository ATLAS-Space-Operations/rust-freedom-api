use reqwest::Response;
use serde::Serialize;

use crate::{api::Api, error::Error};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(skip_serializing)]
    account_id: i32,
    first_name: String,
    last_name: String,
    email: String,
    machine_service: bool,
    roles: Vec<String>,
}

pub struct UserBuilder<'a, C, S> {
    client: &'a C,
    state: S,
}

pub fn new<C>(client: &C) -> UserBuilder<'_, C, NoAccount> {
    UserBuilder {
        client,
        state: NoAccount,
    }
}

pub struct NoAccount;

impl<'a, C> UserBuilder<'a, C, NoAccount> {
    pub fn account_id(self, account_id: impl Into<i32>) -> UserBuilder<'a, C, NoFirstName> {
        UserBuilder {
            client: self.client,
            state: NoFirstName {
                account_id: account_id.into(),
            },
        }
    }
}

pub struct NoFirstName {
    account_id: i32,
}

impl<'a, C> UserBuilder<'a, C, NoFirstName> {
    pub fn first_name(self, first_name: impl Into<String>) -> UserBuilder<'a, C, NoLastName> {
        UserBuilder {
            client: self.client,
            state: NoLastName {
                account_id: self.state.account_id,
                first_name: first_name.into(),
            },
        }
    }
}

pub struct NoLastName {
    account_id: i32,
    first_name: String,
}

impl<'a, C> UserBuilder<'a, C, NoLastName> {
    pub fn last_name(self, last_name: impl Into<String>) -> UserBuilder<'a, C, NoEmail> {
        UserBuilder {
            client: self.client,
            state: NoEmail {
                account_id: self.state.account_id,
                first_name: self.state.first_name,
                last_name: last_name.into(),
            },
        }
    }
}

pub struct NoEmail {
    account_id: i32,
    first_name: String,
    last_name: String,
}

impl<'a, C> UserBuilder<'a, C, NoEmail> {
    pub fn email(self, email: impl Into<String>) -> UserBuilder<'a, C, User> {
        let state = User {
            account_id: self.state.account_id,
            first_name: self.state.first_name,
            last_name: self.state.last_name,
            email: email.into(),
            machine_service: false,
            roles: Vec::new(),
        };

        UserBuilder {
            client: self.client,
            state,
        }
    }
}

impl<'a, C> UserBuilder<'a, C, User> {
    pub fn add_role(mut self, role: impl Into<String>) -> Self {
        self.state.roles.push(role.into());

        self
    }

    pub fn add_roles<I, T>(mut self, roles: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        for role in roles {
            self = self.add_role(role);
        }

        self
    }
}

impl<'a, C> UserBuilder<'a, C, User>
where
    C: Api,
{
    pub async fn send(self) -> Result<Response, Error> {
        let client = self.client;

        let url = client.path_to_url(format!("accounts/{}/newuser", self.state.account_id));
        client.post(url, self.state).await
    }
}
