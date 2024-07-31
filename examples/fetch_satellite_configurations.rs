use freedom_api::prelude::*;
use freedom_config::Config;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;
    let mut client = Client::from_config(config);
    let satellite_configurations = client
        .get_satellite_configurations_by_account_name("ATLAS")
        .filter_map(|result| async move { result.ok() })
        .map(|container| container.into_inner())
        .collect::<Vec<_>>()
        .await;

    println!("{:?}", satellite_configurations);

    Ok(())
}
