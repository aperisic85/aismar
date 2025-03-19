mod ais;
mod client;
mod config;


use crate::config::AisConfig;
use crate::client::connection::AisConnectionManager;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create configuration with multiple endpoints
    let config = AisConfig {
        endpoints: vec![
            "192.168.55.161:4712".into(), // Labinstica
            "192.168.52.161:4712".into(), // VDG
            "192.168.61.161:4712".into(), // ucka
            "192.168.66.161:4712".into(), // osor

        ],
        ..Default::default()
    };

    // Instantiate the connection manager
    //let mut manager = AisConnectionManager::new(config);
    let mut client = client::AisClient::new(config);
    client.run().await?;
    // Start the connection manager
   //manager.start().await?;

    // Wait for Ctrl+C signal to gracefully shut down
    tokio::signal::ctrl_c().await?;
    println!("Shutting down...");

    // Shutdown the connection manager
    //manager.shutdown().await;
    client.shutdown().await;

    Ok(())
}
