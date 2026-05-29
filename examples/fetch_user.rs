use freedom_api::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::from_env()?;
    let me = client.whoami().await?.get_user(&client).await?;

    println!("{:#?}", me);

    Ok(())
}
