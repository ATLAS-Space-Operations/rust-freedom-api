use std::time::Duration;

use freedom_api::prelude::*;
use freedom_config::Config;
use time::OffsetDateTime;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;
    let client = Client::from_config(config);

    let response = client
        .new_task_request()
        .task_type(TaskType::Test)
        .target_date_utc(OffsetDateTime::now_utc() + Duration::from_secs(15 * 60))
        .duration(120)
        .satellite(1016)
        .target_bands([2017, 2019])
        .site(27)
        .configuration(47)
        .send()
        .await?;

    println!("{:#?}", response);

    let text = response.text().await;

    println!("{:#?}", text);

    Ok(())
}
