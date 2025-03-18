// Declare the connection submodule
pub mod connection;
use crate::{ais::decoder, config::AisConfig};
use std::sync::Arc;
use tokio::{net::TcpStream, task::JoinHandle, time};
use connection::AisConnection;
use tokio::sync::mpsc::{self, Sender};

pub struct AisClient {
    config: Arc<AisConfig>,
    handles: Vec<JoinHandle<()>>, // Store handles for each connection task
}

impl AisClient {
    pub fn new(config: AisConfig) -> Self {
        Self {
            config: Arc::new(config),
            handles: Vec::new(),
        }
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        let (tx, mut rx) = mpsc::channel(100); // Create a channel for communication

        // Spawn a connection task for each endpoint
        for endpoint in &self.config.endpoints {
            let endpoint = endpoint.clone();
            let config = self.config.clone();
            let tx_clone = tx.clone(); // Clone sender for each connection

            let handle = tokio::spawn(async move {
                let mut attempt = 0;

                loop {
                    match TcpStream::connect(&endpoint).await {
                        Ok(stream) => {
                            attempt = 0;
                            println!("Connected to {}", endpoint);

                            // Create a new AisConnection and handle it
                            let conn = AisConnection::new(stream, config.clone(), tx_clone.clone());
                            if let Err(e) = conn.handle().await {
                                eprintln!("Connection to {} failed: {}", endpoint, e);
                            }
                        }
                        Err(e) => {
                            attempt += 1;
                            eprintln!("Failed to connect to {}: {}", endpoint, e);

                            if attempt > config.max_reconnect_attempts {
                                eprintln!("Permanently failed to connect to {}", endpoint);
                                break;
                            }

                            time::sleep(config.reconnect_delay).await;
                        }
                    }
                }
            });

            self.handles.push(handle);
        }

        // Monitor received messages from all connections
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                println!("Received decoded message: {}", message);
            }
        });

        Ok(())
    }

    pub async fn shutdown(self) {
        for handle in self.handles {
            handle.abort(); // Gracefully abort all tasks
        }
    }
}
