use freedom_api::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::from_env()?;
    let account = client
        .get_satellite_band_by_id(1)
        .await
        .inspect_err(|error| println!("Failed to get band: {error}"))?
        .get_account(&client)
        .await
        .inspect_err(|error| println!("Failed to get account: {error}"))?;

    println!("{:#?}", account);

    Ok(())
}
