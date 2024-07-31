use freedom_api::prelude::*;
use freedom_config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let atlas_config = Config::from_env()?;
    let atlas_client = Client::from_config(atlas_config);

    let site_from_request: Site = atlas_client
        .get_request_by_id(42)
        .await?
        .into_inner()
        .get_site(&atlas_client)
        .await?;

    println!("{:?}", site_from_request);

    Ok(())
}
