use freedom_api::prelude::*;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::from_env()?;
    let accounts = client
        .get_accounts()
        .filter_map(|result| async move { result.ok() })
        .map(|container| container.into_inner())
        .collect::<Vec<_>>()
        .await;

    println!("{:#?}", accounts);

    Ok(())
}
