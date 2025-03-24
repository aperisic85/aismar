// Declare the connection submodule
pub mod connection;
use crate::config::AisConfig;
use crate::db::database::insert_position_report;
use ais::messages::{AisMessage, position_report};
use connection::AisConnection;
use sqlx::{PgPool, pool};
use std::sync::Arc;
use tokio::sync::mpsc::{self, Sender};
use tokio::{net::TcpStream, task::JoinHandle, time};

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

    pub async fn run(&mut self, pool: Arc<sqlx::PgPool>) -> anyhow::Result<()> {
        let (tx, mut rx) = mpsc::channel(100); // Create a channel for communication
        let pool = pool.clone();
        // Spawn a connection task for each endpoint
        for endpoint in &self.config.endpoints {
            let endpoint = endpoint.clone();
            let config = self.config.clone();
            let tx_clone = tx.clone(); // Clone sender for each connection

            let handle = tokio::spawn(async move {
                let mut attempt = 0;
                println!("Connecting to {}", endpoint);
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
                //println!("Received decoded message: {:?}", message); for debug

                match message {
                    AisMessage::PositionReport(pos) => {
                        let ms = format!(
                            "type: {} MMSI: {} lat: {} lon: {}",
                            pos.message_type,
                            pos.mmsi,
                            pos.latitude.unwrap_or(0.0),
                            pos.longitude.unwrap_or(0.0)
                        );
                        println!("{}", ms);
                        if let Err(e) = insert_position_report(&pool.clone(), pos).await {
                            eprintln!("Failed to insert into database: {}", e);
                        }
                    }
                    _ => (), //println!("[Type s] Unhandled message format",),
                }
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
