use freedom_api::prelude::*;
use freedom_config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;
    let client = Client::from_config(config);

    let response = client.delete_task_request(126171).await?;

    println!("{:#?}", response);

    Ok(())
}
