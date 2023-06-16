use freedom_api::prelude::*;
use freedom_config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;
    let client = Client::from_config(config);
    let tkn = client.get_token_by_satellite(2017, 1016).await?;

    println!("{:?}", tkn);
    Ok(())
}
