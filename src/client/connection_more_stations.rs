// Updated config.rs
#[derive(Clone, Debug)]
pub struct AisConfig {
    pub endpoints: Vec<String>,  // Multiple endpoints to connect to
    pub max_reconnect_attempts: usize,
    pub reconnect_delay: Duration,
    pub read_timeout: Duration,
}

impl Default for AisConfig {
    fn default() -> Self {
        Self {
            endpoints: vec![
                "192.168.51.121:4712".into(),       // Primary source
                "ais.example.com:10110".into(),     // Secondary source
                "aisfeed.openshoring.io:4712".into()// Tertiary source
            ],
            max_reconnect_attempts: 5,
            reconnect_delay: Duration::from_secs(5),
            read_timeout: Duration::from_secs(30),
        }
    }
}

// Modified connection.rs
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub struct AisConnectionManager {
    config: Arc<AisConfig>,
    decoder: Arc<tokio::sync::Mutex<decoder::AisDecoder>>,
    handles: Vec<JoinHandle<()>>,
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
        
        // Spawn a task for each endpoint
        for endpoint in &self.config.endpoints {
            let endpoint = endpoint.clone();
            let config = self.config.clone();
            let decoder = self.decoder.clone();
            
            let handle = tokio::spawn(async move {
                let mut attempt = 0;
                
                loop {
                    match TcpStream::connect(&endpoint).await {
                        Ok(stream) => {
                            attempt = 0;
                            let mut connection = AisConnection::new(stream, config.clone());
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
                // Handle messages from all connections
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

// Modified AisConnection
impl AisConnection {
    pub async fn handle(mut self) -> anyhow::Result<()> {
        let mut buffer = String::new();
        
        loop {
            buffer.clear();
            let read_result = tokio::time::timeout(
                self.config.read_timeout,
                self.stream.read_line(&mut buffer)
            ).await;

            match read_result {
                Ok(Ok(0)) => break, // Clean disconnect
                Ok(Ok(_)) => {
                    let line = buffer.trim_end();
                    let decoder = self.decoder.lock().await;
                    decoder.process(line).await?;
                }
                Ok(Err(e)) => return Err(e).context("Read error"),
                Err(_) => return Err(anyhow::anyhow!("Read timeout")),
            }
        }
        
        Ok(())
    }
}
