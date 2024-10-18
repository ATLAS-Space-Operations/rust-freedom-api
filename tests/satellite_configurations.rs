mod common;

use std::collections::HashMap;

use common::{TestResult, TestingEnv};
use freedom_api::prelude::*;
use futures::StreamExt;
use time::macros::datetime;

fn config(env: &TestingEnv) -> SatelliteConfiguration {
    let mut links = HashMap::new();
    links.insert(
        "self",
        "http://localhost:8080/api/satellite_configurations/812",
    );
    links.insert(
        "configuration",
        "http://localhost:8080/api/satellite_configurations/812",
    );
    links.insert(
        "account",
        "http://localhost:8080/api/satellite_configurations/812/account",
    );
    links.insert(
        "bandDetails",
        "http://localhost:8080/api/satellite_configurations/812/bandDetails",
    );
    let links = env.map_to_links(links);

    SatelliteConfiguration {
        created: datetime!(2022-03-24 19:47:37 UTC),
        modified: Some(datetime!(2024-02-28 19:58:56 UTC)),
        name: String::from("FooBarConfig1"),
        account_name: String::from("ABC Space"),
        orbit: String::from("LEO"),
        notes: String::new(),
        pull_tle: true,
        internal_meta_data: None,
        meta_data: Some(HashMap::new()),
        links,
    }
}

#[tokio::test]
async fn find_all_satellite_configurations() -> TestResult {
    let env = TestingEnv::new();
    let config = config(&env);

    env.get_json_from_file(
        "/satellite_configurations",
        Vec::new(),
        "resources/satellite_configurations_find_all.json",
    );
    let client = Client::from(env);

    let configurations = client
        .get_satellite_configurations()
        .map(|result| result.unwrap().into_inner())
        .collect::<Vec<_>>()
        .await;
    assert_eq!(configurations.len(), 8);
    let first = &configurations[0];
    assert_eq!(first, &config);

    Ok(())
}

#[tokio::test]
async fn find_one_band_by_id() -> TestResult {
    let env = TestingEnv::new();
    let config = config(&env);

    env.get_json_from_file(
        "/satellite_configurations/810",
        Vec::new(),
        "resources/satellite_configurations_find_one_810.json",
    );
    let client = Client::from(env);

    let configuration = client
        .get_satellite_configuration_by_id(810)
        .await?
        .into_inner();
    assert_eq!(configuration, config);

    Ok(())
}

#[tokio::test]
async fn find_one_band_by_name() -> TestResult {
    let env = TestingEnv::new();
    let config = config(&env);

    env.get_json_from_file(
        "/satellite_configurations/search/findOneByName",
        vec![("name", "FooBarConfig1")],
        "resources/satellite_configurations_find_one_810.json",
    );
    let client = Client::from(env);

    let configuration = client
        .get_satellite_configuration_by_name("FooBarConfig1")
        .await?
        .into_inner();
    assert_eq!(configuration, config);

    Ok(())
}
