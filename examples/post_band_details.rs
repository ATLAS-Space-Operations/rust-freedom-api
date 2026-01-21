use std::time::Duration;

use freedom_api::prelude::*;
use freedom_config::Config;
use time::OffsetDateTime;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;
    let client = Client::from_config(config);

    let request = client
        .new_task_request()
        .test_task("idk.bin")
        .target_time_utc(OffsetDateTime::now_utc() + Duration::from_secs(60 * 15))
        .task_duration(120)
        .satellite_id(1016)
        .site_id(27)
        .site_configuration_id(47)
        .band_ids([2017, 2019])
        .send()
        .await?;

    println!("{:#?}", request);

    Ok(())
}
