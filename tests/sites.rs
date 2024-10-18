mod common;

use std::collections::HashMap;

use common::{TestResult, TestingEnv};
use freedom_api::prelude::*;
use freedom_models::azel::Location;
use futures::StreamExt;
use time::macros::datetime;

fn site(env: &TestingEnv) -> Site {
    let mut links = HashMap::new();
    links.insert("self", "http://localhost:8080/api/sites/14");
    links.insert("sites", "http://localhost:8080/api/sites/14");
    links.insert(
        "configurations",
        "http://localhost:8080/api/sites/14/configurations",
    );
    let links = env.map_to_links(links);

    Site {
        created: datetime!(2019-04-22 23:25:40 UTC),
        modified: Some(datetime!(2023-01-26 16:26:48 UTC)),
        internal_meta_data: None,
        name: String::from("LOAG"),
        description: Some(String::from("Los Angeles")),
        location: Location {
            longitude: -2.15,
            latitude: 50.5,
            elevation: 32.652,
        },
        base_fps_port: 20100,
        properties: Some(HashMap::new()),
        links,
    }
}

#[tokio::test]
async fn find_all_sites() -> TestResult {
    let env = TestingEnv::new();
    let site = site(&env);

    env.get_json_from_file("/sites", Vec::new(), "resources/sites_find_all.json");
    let client = Client::from(env);

    let sites = client
        .get_sites()
        .map(|result| result.unwrap().into_inner())
        .collect::<Vec<_>>()
        .await;
    assert_eq!(sites.len(), 1);
    let first = &sites[0];
    assert_eq!(first, &site);

    Ok(())
}

#[tokio::test]
async fn find_one_site_by_id() -> TestResult {
    let env = TestingEnv::new();
    let site = site(&env);

    env.get_json_from_file("/sites/14", Vec::new(), "resources/sites_find_one_14.json");
    let client = Client::from(env);

    let configuration = client.get_site_by_id(14).await?.into_inner();
    assert_eq!(configuration, site);

    Ok(())
}

#[tokio::test]
async fn find_one_site_by_name() -> TestResult {
    let env = TestingEnv::new();
    let site = site(&env);

    env.get_json_from_file(
        "/sites/search/findOneByName",
        vec![("name", "LOAG")],
        "resources/sites_find_one_14.json",
    );
    let client = Client::from(env);

    let configuration = client.get_site_by_name("LOAG").await?.into_inner();
    assert_eq!(configuration, site);

    Ok(())
}
