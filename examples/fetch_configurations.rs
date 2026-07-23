use freedom_api::prelude::*;
use freedom_config::Config;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let atlas_config = Config::from_env()?;
    let atlas_client = Client::from_config(atlas_config);

    let configurations = atlas_client
        .get_site_configurations()
        .collect::<Vec<_>>()
        .await;

    let mut edge = Vec::new();
    for item in configurations {
        let Ok(item) = item else {
            continue;
        };

        let item = item.into_inner();
        let hw = item.hardware.unwrap_or_default();
        for hw in hw {
            if let Some(ref model) = hw.model
                && model == "EDGE"
            {
                let ip = hw.ip.clone().unwrap_or_default();
                let port_comms = hw.port_comms.unwrap_or_default();
                let output = (item.name.clone(), ip, port_comms);
                edge.push(output);
            }
        }
    }

    edge.sort_by(|a, b| a.0.cmp(&b.0));

    println!("{edge:#?}");

    Ok(())
}
