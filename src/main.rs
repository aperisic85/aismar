mod config;
mod client;
mod ais;

use crate::config::AisConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AisConfig {
        endpoint: "ais.xxx.com:1234".into(), // Your AIS provider
        ..Default::default()
    };

    let client = client::AisClient::new(config);
    client.run().await
}
