mod config;
mod client;
mod ais;

use crate::config::AisConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AisConfig {
        endpoint: "192.168.52.161:4712".into(), // Labinstica
        ..Default::default()
    };

    let client = client::AisClient::new(config);
    client.run().await
}
