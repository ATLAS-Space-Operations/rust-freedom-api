mod common;

use std::collections::HashMap;

use common::{TestResult, TestingEnv};
use freedom_api::prelude::*;
use futures::StreamExt;
use time::macros::datetime;

fn sat(env: &TestingEnv) -> Satellite {
    let mut links = HashMap::new();
    links.insert("self", "http://localhost:8080/api/satellites/710");
    links.insert("satellites", "http://localhost:8080/api/satellites/710");
    links.insert(
        "upcomingVisibilities",
        "http://localhost:8080/api/satellites/710/upcomingVisibilities",
    );
    links.insert(
        "account",
        "http://localhost:8080/api/satellites/710/account",
    );
    links.insert(
        "orbitInfo",
        "http://localhost:8080/api/satellites/710/orbitInfo",
    );
    links.insert(
        "configuration",
        "http://localhost:8080/api/satellites/710/configuration",
    );
    let links = env.map_to_links(links);

    Satellite {
        created: datetime!(2022-03-24 19:48:19 UTC),
        modified: Some(datetime!(2024-10-18 00:00:53 UTC)),
        name: String::from("FooBar 6"),
        description: String::from("FooBar 6 Demo Satellite"),
        norad_cat_id: Some(100),
        tle: Some(TwoLineElement {
            line1: String::from("TLE"),
            line2: String::from("TLE"),
        }),
        internal_meta_data: None,
        account_name: String::from("ABC Space"),
        meta_data: Some(HashMap::new()),
        links,
    }
}

#[tokio::test]
async fn find_all_satellites() -> TestResult {
    let env = TestingEnv::new();
    let sat = sat(&env);

    env.get_json_from_file(
        "/satellites",
        Vec::new(),
        "resources/satellite_find_all.json",
    );
    let client = Client::from(env);

    let satellites = client
        .get_satellites()
        .map(|result| result.unwrap().into_inner())
        .collect::<Vec<_>>()
        .await;
    assert_eq!(satellites.len(), 14);
    let first = &satellites[0];
    assert_eq!(first, &sat);

    Ok(())
}

#[tokio::test]
async fn find_one_satellite_by_id() -> TestResult {
    let env = TestingEnv::new();
    let sat = sat(&env);

    env.get_json_from_file(
        "/satellites/710",
        Vec::new(),
        "resources/satellite_find_one_710.json",
    );
    let client = Client::from(env);

    let satellite = client.get_satellite_by_id(710).await?.into_inner();
    assert_eq!(satellite, sat);

    Ok(())
}

#[tokio::test]
async fn find_one_satellite_by_name() -> TestResult {
    let env = TestingEnv::new();
    let sat = sat(&env);

    env.get_json_from_file(
        "/satellites/findOneByName",
        vec![("name", "FooBar 6")],
        "resources/satellite_find_one_710.json",
    );
    let client = Client::from(env);

    let satellite = client.get_satellite_by_name("FooBar 6").await?.into_inner();
    assert_eq!(satellite, sat);

    Ok(())
}
