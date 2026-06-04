use freedom_api::{GatewayApi, prelude::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::from_env()?;

    let latest_version = client.get_gateway_latest_version().await?;

    println!(
        "The latest Gateway version is: {}",
        latest_version.latest_tag
    );

    Ok(())
}
