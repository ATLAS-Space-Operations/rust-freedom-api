use std::pin::Pin;

use async_trait::async_trait;
use futures_core::Stream;
use mockall::{mock, predicate::*};

use crate::{client::Inner, prelude::FreedomApi};
use url::Url;

mock! {
    pub FreedomApi {}
    #[async_trait]
    impl FreedomApi for FreedomApi {
        async fn get<T>(&mut self, url: Url) -> Result<T, crate::error::Error>
        where
            T: serde::de::DeserializeOwned;

        async fn post<S, T>(&mut self, url: Url) -> Result<T, crate::error::Error>
        where
            S: serde::Serialize,
            T: serde::de::DeserializeOwned;

        async fn get_account_by_id(
            &mut self,
            account_id: i32,
        ) -> Result<Self::Container<Account>, crate::error::Error>;

        fn get_accounts(
            &mut self,
        ) -> Pin<Box<dyn Stream<Item = Result<Self::Container<Account>, crate::error::Error>> + '_>>;

        fn get_env(&self) -> &Environment;

        fn get_paginated<T>(
            &mut self,
            head_url: Url,
        ) -> Pin<Box<dyn Stream<Item = Result<Self::Container<T>, crate::error::Error>> + '_>>
        where
            T: std::fmt::Debug + DeserializeOwned + 'static;

        async fn get_request_by_task_id(
            &mut self,
            task_id: i32,
        ) -> Result<Self::Container<TaskRequest>, crate::error::Error>;

        fn get_requests(
            &mut self,
        ) -> Pin<Box<dyn Stream<Item = Result<Self::Container<TaskRequest>, crate::error::Error>> + '_>>;

        fn get_requests_by_account_and_target_date_between<T>(
            &mut self,
            account_uri: T,
            start: OffsetDateTime,
            end: OffsetDateTime,
        ) -> Pin<Box<dyn Stream<Item = Result<Self::Container<TaskRequest>, crate::error::Error>> + '_>>
        where
            T: AsRef<str> + Send;

        fn get_requests_by_account_and_upcoming_today(
            &mut self,
        ) -> Pin<Box<dyn Stream<Item = Result<Self::Container<TaskRequest>, crate::error::Error>> + '_>>;

        fn get_requests_by_configuration<T>(
            &mut self,
            configuration_uri: T,
        ) -> Pin<Box<dyn Stream<Item = Result<Self::Container<TaskRequest>, crate::error::Error>> + '_>>
        where
            T: AsRef<str> + Send;

        async fn get_requests_by_configuration_and_satellite_names_and_target_date_between<T, I, S>(
            &mut self,
            configuration_uri: T,
            satellites: I,
            start: OffsetDateTime,
            end: OffsetDateTime,
        ) -> Result<Self::Container<Vec<TaskRequest>>, crate::error::Error>
        where
            T: AsRef<str> + Send,
            I: IntoIterator<Item = S> + Send,
            S: AsRef<str> + Send;

        async fn get_requests_by_configuration_and_target_date_between<T>(
            &mut self,
            configuration_uri: T,
            start: OffsetDateTime,
            end: OffsetDateTime,
        ) -> Result<Self::Container<Vec<TaskRequest>>, crate::error::Error>
        where
            T: AsRef<str> + Send;

        async fn get_requests_by_ids<I, S>(
            &mut self,
            ids: I,
        ) -> Result<Self::Container<Vec<TaskRequest>>, crate::error::Error>
        where
            I: IntoIterator<Item = S> + Send,
            S: AsRef<str> + Send;

        fn get_requests_by_overlapping_public(
            &mut self,
            start: OffsetDateTime,
            end: OffsetDateTime,
        ) -> Pin<Box<dyn Stream<Item = Result<Self::Container<TaskRequest>, crate::error::Error>> + '_>>;

        fn get_requests_by_satellite_name<T>(
            &mut self,
            satellite_name: T,
        ) -> Pin<Box<dyn Stream<Item = Result<Self::Container<TaskRequest>, crate::error::Error>> + '_>>
        where
            T: AsRef<str> + Send;

        async fn get_requests_by_satellite_name_and_target_date_between<T>(
            &mut self,
            satellite_name: T,
            start: OffsetDateTime,
            end: OffsetDateTime,
        ) -> Result<Self::Container<Vec<TaskRequest>>, crate::error::Error>
        where
            T: AsRef<str> + Send;

        fn get_requests_by_status<T>(
            &mut self,
            status: T,
        ) -> Result<
            Pin<Box<dyn Stream<Item = Result<Self::Container<TaskRequest>, crate::error::Error>> + '_>>,
            crate::error::Error,
        >
        where
            T: TryInto<TaskStatusType> + Send,
            crate::error::Error: From<<T as TryInto<TaskStatusType>>::Error>;

        fn get_requests_by_status_and_account_and_target_date_between<T, U>(
            &mut self,
            status: T,
            account_uri: U,
            start: OffsetDateTime,
            end: OffsetDateTime,
        ) -> Pin<Box<dyn Stream<Item = Result<Self::Container<TaskRequest>, crate::error::Error>> + '_>>
        where
            T: AsRef<str> + Send,
            U: AsRef<str> + Send;

        async fn get_requests_by_type_and_target_date_between<T>(
            &mut self,
            typ: T,
            start: OffsetDateTime,
            end: OffsetDateTime,
        ) -> Result<Self::Container<Vec<TaskRequest>>, crate::error::Error>
        where
            T: TryInto<TaskType> + Send,
            crate::error::Error: From<<T as TryInto<TaskType>>::Error>;

        async fn get_requests_passed_today(
            &mut self,
        ) -> Result<Self::Container<Vec<TaskRequest>>, crate::error::Error>;

        async fn get_requests_upcoming_today(
            &mut self,
        ) -> Result<Self::Container<Vec<TaskRequest>>, crate::error::Error>;

        fn get_satellites(
            &mut self,
        ) -> Pin<Box<dyn Stream<Item = Result<Self::Container<Satellite>, crate::error::Error>> + '_>>;

        async fn get_task_by_id(
            &mut self,
            task_id: i32,
        ) -> Result<Self::Container<Task>, crate::error::Error>;

        async fn get_tasks_by_account_and_pass_overlapping<T>(
            &mut self,
            account_uri: T,
            start: OffsetDateTime,
            end: OffsetDateTime,
        ) -> Result<Self::Container<Vec<Task>>, crate::error::Error>
        where
            T: AsRef<str> + Send;

        async fn get_tasks_by_account_and_satellite_and_band_and_pass_overlapping<T, U, V>(
            &mut self,
            account_uri: T,
            satellite_config_uri: U,
            band: V,
            start: OffsetDateTime,
            end: OffsetDateTime,
        ) -> Result<Self::Container<Vec<Task>>, crate::error::Error>
        where
            T: AsRef<str> + Send,
            U: AsRef<str> + Send,
            V: AsRef<str> + Send;

        async fn get_tasks_by_account_and_site_configuration_and_band_and_pass_overlapping<T, U, V>(
            &mut self,
            account_uri: T,
            site_config_uri: U,
            band: V,
            start: OffsetDateTime,
            end: OffsetDateTime,
        ) -> Result<Self::Container<Vec<Task>>, crate::error::Error>
        where
            T: AsRef<str> + Send,
            U: AsRef<str> + Send,
            V: AsRef<str> + Send;

        async fn get_tasks_by_pass_overlapping(
            &mut self,
            start: OffsetDateTime,
            end: OffsetDateTime,
        ) -> Result<Self::Container<Vec<Task>>, crate::error::Error>;

        async fn get_tasks_passed_today(
            &mut self,
        ) -> Result<Self::Container<Vec<Task>>, crate::error::Error>;

        async fn get_tasks_upcoming_today(
            &mut self,
        ) -> Result<Self::Container<Vec<Task>>, crate::error::Error>;

        fn get_users(
            &mut self,
        ) -> Pin<Box<dyn Stream<Item = Result<Self::Container<User>, crate::error::Error>> + '_>>;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn idk() {}
}
