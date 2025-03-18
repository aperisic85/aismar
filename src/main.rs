mod ais;
mod client;
mod config;

use crate::config::AisConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Update configuration to support multiple endpoints
    let config = AisConfig {
        endpoints: vec![
            "192.168.52.161:4712".into(), // Labinstica
            "192.168.54.162:4712".into(), // Vidova gora
            "192.168.61.162:4712".into(), // Ucka
        ],
        ..Default::default()
    };

    let mut client = client::AisClient::new(config); // Updated to support multiple endpoints
    client.run().await?;

    tokio::signal::ctrl_c().await?; // Handle shutdown gracefully
    client.shutdown().await;

    Ok(())
}
