#![allow(unused, dead_code)]

use std::{collections::HashMap, path::Path};

use freedom_api::Client;
use freedom_config::Config;
use httpmock::{prelude::*, Mock};
use url::Url;

pub type TestResult = std::result::Result<(), Box<dyn std::error::Error + 'static + Send + Sync>>;

pub struct TestingEnv {
    server: MockServer,
}

impl std::fmt::Debug for TestingEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestingEnv").finish()
    }
}

impl AsRef<str> for TestingEnv {
    fn as_ref(&self) -> &str {
        "TestingEnv"
    }
}

impl TestingEnv {
    pub fn map_to_links(&self, map: HashMap<&str, &str>) -> HashMap<String, Url> {
        map.into_iter()
            .map(|(key, val)| {
                let val = val.replace("localhost:8080", &format!("localhost:{}", self.port()));
                (key.to_string(), Url::parse(&val).unwrap())
            })
            .collect()
    }

    pub fn new() -> Self {
        let server = MockServer::start();
        Self { server }
    }

    pub fn get_json_from_file(
        &self,
        path: &str,
        query: Vec<(&str, &str)>,
        file: impl AsRef<Path>,
    ) -> Mock {
        let file = std::fs::read(file).unwrap();
        let file = String::from_utf8(file).unwrap();
        let file = file.replace("localhost:8080", &format!("localhost:{}", self.port()));
        eprintln!("{}", file);
        self.mock(|mut when, then| {
            when = when.method(GET).path(path);
            for (name, value) in query {
                when = when.query_param(name, value);
            }

            then.status(200)
                .header("content-type", "application/json")
                .body(file);
        })
    }
}

impl std::ops::Deref for TestingEnv {
    type Target = MockServer;

    fn deref(&self) -> &Self::Target {
        &self.server
    }
}

impl std::ops::DerefMut for TestingEnv {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.server
    }
}

impl From<TestingEnv> for Client {
    fn from(value: TestingEnv) -> Self {
        let config = Config::builder()
            .environment(value)
            .key("")
            .secret("")
            .build()
            .unwrap();

        Client::from_config(config)
    }
}

impl Default for TestingEnv {
    fn default() -> Self {
        Self::new()
    }
}

impl freedom_config::Env for TestingEnv {
    fn from_str(_val: &str) -> Option<Self>
    where
        Self: Sized,
    {
        Some(Self::new())
    }

    fn fps_host(&self) -> &str {
        todo!()
    }

    fn freedom_entrypoint(&self) -> Url {
        let url = self.server.base_url();
        Url::parse(&url).unwrap()
    }
}
