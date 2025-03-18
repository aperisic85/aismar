// Declare the connection submodule
pub mod connection;

use crate::{ais::decoder, config::AisConfig};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::time;

pub struct AisClient {
    config: Arc<AisConfig>,
}

impl AisClient {
    pub fn new(config: AisConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let mut attempt = 0;

        loop {
            match TcpStream::connect(&self.config.endpoint).await {
                Ok(stream) => {
                    attempt = 0;
                    // Use self::connection instead of client::connection
                    let conn = self::connection::AisConnection::new(stream, self.config.clone());

                    if let Err(e) = conn.handle().await {
                        eprintln!("Connection error: {}", e);
                    }
                }
                Err(e) => {
                    // ... rest unchanged ...
                    attempt += 1;
                    if attempt > self.config.max_reconnect_attempts {
                        return Err(e.into());
                    }

                    time::sleep(self.config.reconnect_delay).await;
                }
            }
        }
    }
}
