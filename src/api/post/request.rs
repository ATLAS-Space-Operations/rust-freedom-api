use freedom_models::task::TaskType;
use reqwest::Response;
use serde::Serialize;
use time::OffsetDateTime;

use crate::{api::Api, error::Error};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskRequest {
    #[serde(rename(serialize = "type"))]
    typ: TaskType,
    site: String,
    satellite: String,
    configuration: String,
    target_bands: Vec<String>,
    target_date: String,
    duration: u64,
    minimum_duration: Option<u64>,
    hours_of_flex: Option<u8>,
    test_file: Option<String>,
    #[serde(rename(serialize = "override"))]
    with_override: Option<String>,
}

pub struct TaskRequestBuilder<'a, C, S> {
    pub(crate) client: &'a C,
    state: S,
}

pub fn new<C>(client: &C) -> TaskRequestBuilder<'_, C, NoType> {
    TaskRequestBuilder {
        client,
        state: NoType,
    }
}

pub trait TaskInner {
    fn adjust(&mut self, request: &mut TaskRequest);
}

impl TaskInner for TestTask {
    fn adjust(&mut self, request: &mut TaskRequest) {
        request.typ = TaskType::Test;
        request.test_file = Some(std::mem::take(&mut self.test_file));
    }
}

impl TaskInner for FlexTask {
    fn adjust(&mut self, request: &mut TaskRequest) {
        let kind = match self.kind {
            FlexTaskKind::Before => TaskType::Before,
            FlexTaskKind::After => TaskType::After,
            FlexTaskKind::Around => TaskType::Around,
        };

        request.typ = kind;
        request.hours_of_flex = Some(self.hours_of_flex);
    }
}

pub struct NoType;

pub struct TestTask {
    test_file: String,
}

pub enum FlexTaskKind {
    Before,
    After,
    Around,
}

pub struct FlexTask {
    kind: FlexTaskKind,
    hours_of_flex: u8,
}

pub struct ExactTask;

impl TaskInner for ExactTask {
    fn adjust(&mut self, request: &mut TaskRequest) {
        request.typ = TaskType::Exact;
    }
}

impl<'a, C> TaskRequestBuilder<'a, C, NoType> {
    pub fn exact_task(self) -> TaskRequestBuilder<'a, C, NoTime<ExactTask>> {
        TaskRequestBuilder {
            client: self.client,
            state: NoTime { kind: ExactTask },
        }
    }

    pub fn flex_task(
        self,
        kind: FlexTaskKind,
        hours_of_flex: u8,
    ) -> TaskRequestBuilder<'a, C, NoTime<FlexTask>> {
        TaskRequestBuilder {
            client: self.client,
            state: NoTime {
                kind: FlexTask {
                    kind,
                    hours_of_flex,
                },
            },
        }
    }

    pub fn flex_task_after(self, hours_of_flex: u8) -> TaskRequestBuilder<'a, C, NoTime<FlexTask>> {
        self.flex_task(FlexTaskKind::After, hours_of_flex)
    }

    pub fn flex_task_around(
        self,
        hours_of_flex: u8,
    ) -> TaskRequestBuilder<'a, C, NoTime<FlexTask>> {
        self.flex_task(FlexTaskKind::Around, hours_of_flex)
    }

    pub fn flex_task_before(
        self,
        hours_of_flex: u8,
    ) -> TaskRequestBuilder<'a, C, NoTime<FlexTask>> {
        self.flex_task(FlexTaskKind::Before, hours_of_flex)
    }

    pub fn test_task(
        self,
        test_file: impl Into<String>,
    ) -> TaskRequestBuilder<'a, C, NoTime<TestTask>> {
        TaskRequestBuilder {
            client: self.client,
            state: NoTime {
                kind: TestTask {
                    test_file: test_file.into(),
                },
            },
        }
    }
}

pub struct NoTime<T> {
    kind: T,
}

impl<'a, C, T> TaskRequestBuilder<'a, C, NoTime<T>> {
    pub fn target_time_utc(self, time: OffsetDateTime) -> TaskRequestBuilder<'a, C, NoDuration<T>> {
        TaskRequestBuilder {
            client: self.client,
            state: NoDuration {
                kind: self.state.kind,
                time,
            },
        }
    }
}

pub struct NoDuration<T> {
    kind: T,
    time: OffsetDateTime,
}

impl<'a, C, T> TaskRequestBuilder<'a, C, NoDuration<T>> {
    pub fn task_duration(self, seconds: u64) -> TaskRequestBuilder<'a, C, NoSatellite<T>> {
        TaskRequestBuilder {
            client: self.client,
            state: NoSatellite {
                kind: self.state.kind,
                time: self.state.time,
                duration: seconds,
            },
        }
    }
}

pub struct NoSatellite<T> {
    kind: T,
    time: OffsetDateTime,
    duration: u64,
}

impl<'a, C, T> TaskRequestBuilder<'a, C, NoSatellite<T>>
where
    C: Api,
{
    pub fn satellite_id(self, id: impl Into<i32>) -> TaskRequestBuilder<'a, C, NoSite<T>> {
        let satellite = self
            .client
            .path_to_url(format!("satellites/{}", id.into()))
            .to_string();

        self.satellite_url(satellite)
    }
}

impl<'a, C, T> TaskRequestBuilder<'a, C, NoSatellite<T>> {
    pub fn satellite_url(self, url: impl Into<String>) -> TaskRequestBuilder<'a, C, NoSite<T>> {
        TaskRequestBuilder {
            client: self.client,
            state: NoSite {
                kind: self.state.kind,
                time: self.state.time,
                duration: self.state.duration,
                satellite: url.into(),
            },
        }
    }
}

pub struct NoSite<T> {
    kind: T,
    time: OffsetDateTime,
    duration: u64,
    satellite: String,
}

impl<'a, C, T> TaskRequestBuilder<'a, C, NoSite<T>>
where
    C: Api,
{
    pub fn site_id(self, id: impl Into<i32>) -> TaskRequestBuilder<'a, C, NoConfig<T>> {
        let site = self
            .client
            .path_to_url(format!("sites/{}", id.into()))
            .to_string();

        self.site_url(site)
    }
}

impl<'a, C, T> TaskRequestBuilder<'a, C, NoSite<T>> {
    pub fn site_url(self, url: impl Into<String>) -> TaskRequestBuilder<'a, C, NoConfig<T>> {
        TaskRequestBuilder {
            client: self.client,
            state: NoConfig {
                kind: self.state.kind,
                time: self.state.time,
                duration: self.state.duration,
                satellite: self.state.satellite,
                site: url.into(),
            },
        }
    }
}

pub struct NoConfig<T> {
    kind: T,
    time: OffsetDateTime,
    duration: u64,
    satellite: String,
    site: String,
}

impl<'a, C, T> TaskRequestBuilder<'a, C, NoConfig<T>>
where
    C: Api,
{
    pub fn site_configuration_id(self, id: impl Into<i32>) -> TaskRequestBuilder<'a, C, NoBand<T>> {
        let configuration = self
            .client
            .path_to_url(format!("configurations/{}", id.into()))
            .to_string();

        self.site_configuration_url(configuration)
    }
}

impl<'a, C, T> TaskRequestBuilder<'a, C, NoConfig<T>> {
    pub fn site_configuration_url(
        self,
        url: impl Into<String>,
    ) -> TaskRequestBuilder<'a, C, NoBand<T>> {
        TaskRequestBuilder {
            client: self.client,
            state: NoBand {
                kind: self.state.kind,
                time: self.state.time,
                duration: self.state.duration,
                satellite: self.state.satellite,
                site: self.state.site,
                configuration: url.into(),
            },
        }
    }
}

pub struct NoBand<T> {
    kind: T,
    time: OffsetDateTime,
    duration: u64,
    satellite: String,
    site: String,
    configuration: String,
}

impl<'a, C, T> TaskRequestBuilder<'a, C, NoBand<T>>
where
    T: TaskInner,
{
    pub fn band_ids(
        self,
        ids: impl IntoIterator<Item = i32>,
    ) -> TaskRequestBuilder<'a, C, TaskRequest>
    where
        C: Api,
    {
        let client = self.client;
        let bands = ids.into_iter().map(|id| {
            client
                .path_to_url(format!("satellite_bands/{}", id))
                .to_string()
        });

        self.band_urls(bands)
    }

    pub fn band_urls(
        mut self,
        urls: impl IntoIterator<Item = String>,
    ) -> TaskRequestBuilder<'a, C, TaskRequest> {
        use time::macros::format_description;
        let item = format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]Z");

        let target_date = self.state.time.format(item).unwrap();
        let target_bands: Vec<_> = urls.into_iter().collect();

        let mut state = TaskRequest {
            typ: TaskType::After, // This is overwritten in `adjust`
            site: self.state.site,
            satellite: self.state.satellite,
            configuration: self.state.configuration,
            target_bands,
            target_date,
            duration: self.state.duration,
            minimum_duration: Some(self.state.duration),
            hours_of_flex: None,
            test_file: None,
            with_override: None,
        };

        self.state.kind.adjust(&mut state);

        TaskRequestBuilder {
            client: self.client,
            state,
        }
    }
}

impl<C> TaskRequestBuilder<'_, C, TaskRequest> {
    pub fn task_minimum_duration(mut self, duration: u64) -> Self {
        self.state.minimum_duration = Some(duration);
        self
    }

    pub fn override_url(mut self, url: impl Into<String>) -> Self {
        self.state.with_override = Some(url.into());
        self
    }
}

impl<C> TaskRequestBuilder<'_, C, TaskRequest>
where
    C: Api,
{
    pub fn override_id(self, id: impl Into<i32>) -> Self {
        let override_url = self
            .client
            .path_to_url(format!("overrides/{}", id.into()))
            .to_string();

        self.override_url(override_url)
    }

    pub async fn send(self) -> Result<Response, Error> {
        let client = self.client;

        let url = client.path_to_url("requests");
        client.post(url, self.state).await
    }
}
