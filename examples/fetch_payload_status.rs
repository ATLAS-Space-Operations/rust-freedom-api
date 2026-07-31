use freedom_api::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::builder()
        .environment(Test)
        .key_from_env()?
        .secret_from_env()?
        .build()?;

    let client = Client::from_config(config);
    let status = client
        .get_payload_status(288818, "TEST_RX_100Mbps", None)
        .await?;

    println!("{:#?}", status);

    Ok(())
}
