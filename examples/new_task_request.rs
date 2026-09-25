use freedom_api::prelude::*;
use freedom_config::Config;
use time::{Duration, OffsetDateTime};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;
    let client = Client::from_config(config);
    let start = OffsetDateTime::now_utc() + Duration::minutes(3);
    let response = client
        .new_task_request()
        .test_task("MY_TEST_FILE.bin")
        .target_time_utc(start)
        .task_duration(2 * 60)
        .satellite_id(1)
        .site_id(1)
        .site_configuration_id(1)
        .band_ids([1])
        .send()
        .await?;

    println!("{:?}", response);
    Ok(())
}
