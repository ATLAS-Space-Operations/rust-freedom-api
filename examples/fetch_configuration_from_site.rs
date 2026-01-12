use freedom_api::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::from_env()?;
    let configurations = client
        .get_site_by_id(12)
        .await?
        .get_site_configurations(&client)
        .await?;

    println!("{:#?}", configurations);

    Ok(())
}
