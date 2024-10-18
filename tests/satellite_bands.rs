mod common;

use std::collections::HashMap;

use common::{TestResult, TestingEnv};
use freedom_api::prelude::*;
use freedom_models::band::{Band, BandType, IoConfiguration, IoHardware};
use futures::StreamExt;
use time::macros::datetime;

fn band(env: &TestingEnv) -> Band {
    let mut links = HashMap::new();
    links.insert("self", "http://localhost:8080/api/satellite_bands/1573");
    links.insert("bands", "http://localhost:8080/api/satellite_bands/1573");
    links.insert(
        "account",
        "http://localhost:8080/api/satellite_bands/1573/account",
    );
    let links = env.map_to_links(links);

    Band {
        created: datetime!(2022-03-24 19:47:18 UTC),
        modified: Some(datetime!(2023-10-11 19:37:46 UTC)),
        name: String::from("FooBarBand1"),
        typ: Some(BandType::Receive),
        frequency_mghz: 1000.0,
        default_band_width_mghz: 1000.0,
        io_configuration: IoConfiguration {
            start_hex_pattern: Some(String::new()),
            end_hex_pattern: Some(String::new()),
            strip_pattern: false,
            io_hardware: Some(IoHardware::Modem),
        },
        manual_transmit_control: Some(false),
        account_name: Some(String::from("ABC Space")),
        links,
    }
}

#[tokio::test]
async fn find_all_bands() -> TestResult {
    let env = TestingEnv::new();
    let band = band(&env);

    env.get_json_from_file(
        "/satellite_bands",
        Vec::new(),
        "resources/satellite_bands_find_all.json",
    );
    let client = Client::from(env);

    let bands = client
        .get_satellite_bands()
        .map(|result| result.unwrap().into_inner())
        .collect::<Vec<_>>()
        .await;
    assert_eq!(bands.len(), 6);
    let first = &bands[0];
    assert_eq!(first, &band);

    Ok(())
}

#[tokio::test]
async fn find_one_band_by_id() -> TestResult {
    let env = TestingEnv::new();
    let band = band(&env);

    env.get_json_from_file(
        "/satellite_bands/1573",
        Vec::new(),
        "resources/satellite_bands_find_one_1573.json",
    );
    let client = Client::from(env);

    let band_recv = client.get_satellite_band_by_id(1573).await?.into_inner();
    assert_eq!(band_recv, band);

    Ok(())
}

#[tokio::test]
async fn find_one_band_by_name() -> TestResult {
    let env = TestingEnv::new();
    let band = band(&env);

    env.get_json_from_file(
        "/satellite_bands/search/findOneByName",
        vec![("name", "FooBar1")],
        "resources/satellite_bands_find_one_1573.json",
    );
    let client = Client::from(env);

    let band_recv = client
        .get_satellite_band_by_name("FooBar1")
        .await?
        .into_inner();
    assert_eq!(band_recv, band);

    Ok(())
}
