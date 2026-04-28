use freedom_api::prelude::*;
use freedom_config::Config;
use time::macros::datetime;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;
    let client = Client::from_config(config);
    let response = client
        .new_task_request()
        .exact_task()
        .target_time_utc(datetime!(2026-03-16 23:00:00 UTC))
        .task_duration(10 * 60)
        .satellite_id(1016)
        .site_id(75)
        .site_configuration_id(85)
        .band_ids([2017])
        .send()
        .await?;

    println!("{:?}", response);
    Ok(())
}
