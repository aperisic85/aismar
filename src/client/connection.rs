use crate::{ais::decoder, config::AisConfig};
use anyhow::Context;
use serde::de;
use std::sync::Arc;
use tokio::io::AsyncBufReadExt;
use tokio::{io::BufReader, net::TcpStream, time};
use tokio::task::{JoinHandle, JoinError};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

pub struct AisConnection {
    stream: BufReader<TcpStream>,
    config: Arc<AisConfig>,
    decoder: Arc<Mutex<decoder::AisDecoder>>,
    tx: Sender<String>, // Channel to send decoded results
}

pub struct AisConnectionManager {
    config: Arc<AisConfig>,
    decoder: Arc<tokio::sync::Mutex<decoder::AisDecoder>>,
    handles: Vec<JoinHandle<()>>,
}

impl AisConnection {
    pub fn new(stream: TcpStream, config: Arc<AisConfig>, tx: Sender<String>) -> Self {
        Self {
            stream: BufReader::new(stream),
            config,
            decoder: Arc::new(Mutex::new(decoder::AisDecoder::new())),
            tx,
        }
    }

    pub async fn handle(mut self) -> anyhow::Result<()> {
        let mut buffer = String::new();

        loop {
            buffer.clear();
            let read_result = tokio::time::timeout(
                self.config.read_timeout,
                self.stream.read_line(&mut buffer),
            )
            .await;

            match read_result {
                Ok(Ok(0)) => break, // Clean disconnect
                Ok(Ok(_)) => {
                    let line = buffer.trim_end();
                    let mut decoder = self.decoder.lock().await;
                    match decoder.process(line).await {
                        Ok(decoded) => {
                            if self.tx.send(decoded).await.is_err() {
                                eprintln!("Failed to send decoded message");
                            }
                        }
                        Err(e) => eprintln!("Decoding error: {}", e),
                    }
                }
                Ok(Err(e)) => return Err(e).context("Read error"),
                Err(_) => return Err(anyhow::anyhow!("Read timeout")),
            } 
       }

        Ok(())
    }
}

impl AisConnectionManager {
    pub fn new(config: AisConfig) -> Self {
        Self {
            config: Arc::new(config),
            decoder: Arc::new(tokio::sync::Mutex::new(decoder::AisDecoder::new())),
            handles: Vec::new(),
        }
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        let (tx, mut rx) = mpsc::channel(100);
    
        for endpoint in &self.config.endpoints {
            let endpoint = endpoint.clone();
            let config = self.config.clone();
            let decoder = self.decoder.clone();
            let tx_clone = tx.clone(); // Clone sender for each connection
    
            let handle = tokio::spawn(async move {
                let mut attempt = 0;
    
                loop {
                    match TcpStream::connect(&endpoint).await {
                        Ok(stream) => {
                            attempt = 0;
                            let connection = AisConnection::new(stream, config.clone(), tx_clone.clone());
                            if let Err(e) = connection.handle().await {
                                eprintln!("Connection to {} failed: {}", endpoint, e);
                            }
                        }
                        Err(e) => {
                            attempt += 1;
                            if attempt > config.max_reconnect_attempts {
                                eprintln!("Permanently failed to connect to {}", endpoint);
                                break;
                            }
                            tokio::time::sleep(config.reconnect_delay).await;
                        }
                    }
                }
            });
    
            self.handles.push(handle);
        }
    
        // Monitor tasks
        tokio::spawn(async move {
            while let Some(result) = rx.recv().await {
                println!("Received: {:?}", result);
            }
        });
    
        Ok(())
    }

    pub async fn shutdown(self) {
        for handle in self.handles {
            handle.abort();
        }
    }
}