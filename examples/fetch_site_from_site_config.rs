use freedom_api::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::from_env()?;
    let site = client
        .get_site_configuration_by_id(19)
        .await?
        .get_site(&client)
        .await?;

    println!("{:#?}", site);

    Ok(())
}
