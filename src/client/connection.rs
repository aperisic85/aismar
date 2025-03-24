use crate::{ais::decoder, config::AisConfig};
use ais::messages::AisMessage;
use anyhow::Context;
use serde::de;
use std::sync::Arc;
use tokio::io::AsyncBufReadExt;
use tokio::{io::BufReader, net::TcpStream, time};
use tokio::task::{JoinHandle, JoinError};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use ais::AisFragments;

pub struct AisConnection {
    stream: BufReader<TcpStream>,
    config: Arc<AisConfig>,
    decoder: Arc<Mutex<decoder::AisDecoder>>,
    tx: tokio::sync::mpsc::Sender<AisMessage>, // Channel to send decoded results
}

pub struct AisConnectionManager {
    config: Arc<AisConfig>,
    decoder: Arc<tokio::sync::Mutex<decoder::AisDecoder>>,
    handles: Vec<JoinHandle<()>>,
}

impl AisConnection {
    pub fn new(stream: TcpStream, config: Arc<AisConfig>, tx: Sender<AisMessage>) -> Self {
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
                self.stream.read_line(&mut buffer)
            ).await;
    
            match read_result {
                Ok(Ok(0)) => break, // Clean disconnect
                Ok(Ok(_)) => {
                    let line = buffer.trim_end();
                    let mut decoder = self.decoder.lock().await;
                    match decoder.parser.parse(line.as_bytes(), true) {
                        Ok(AisFragments::Complete(sentence)) => {
                            if let Some(msg) = sentence.message {
                                if let Err(e) = decoder.handle_message(msg, line, self.tx.to_owned()).await {
                                    eprintln!("Message handling error: {}", e);
                                }
                            }
                        }
                        Err(e) => eprintln!("Parsing error: {}", e),
                        _ => {} // Handle incomplete fragments if needed
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

                /* match msg {
            AisMessage::PositionReport(pos) => {
                 println!("[Type {}] Vessel {}: {:?} {:?} | SOG: {} kt | Nav Status: {:?}, name {}",
                    pos.message_type,
                    pos.mmsi,
                    pos.latitude.unwrap_or(0.0),
                    pos.longitude.unwrap_or(0.0),
                    pos.speed_over_ground.unwrap_or(0.0),
                    pos.navigation_status.unwrap_or(ais::messages::position_report::NavigationStatus::Unknown(2)),
                    pos.name()

                   
                ); 
                
            }
            AisMessage::BaseStationReport(bs) => {
                /*  println!("[Type {}] Base Station {}: {:?} UTC",
                    bs.message_type,
                    bs.mmsi,
                    bs.hour
                ); */
            }
            AisMessage::StaticAndVoyageRelatedData(sdv) => {
                /*  println!("[Type {}] Vessel {}: {} → {}",
                    sdv.message_type,
                    sdv.mmsi,
                    sdv.vessel_name,
                    sdv.destination
                ); */
            }
            AisMessage::StandardClassBPositionReport(scb) => {
                /*  println!("[Type {}] Class B {}: {:?} {:?} COG: {}",
                    scb.message_type,
                    scb.mmsi,
                    scb.latitude.unwrap_or(0.0),
                    scb.longitude.unwrap_or(0.0),
                    scb.course_over_ground.unwrap_or(0.0)
                ); */
            }
            AisMessage::ExtendedClassBPositionReport(ecb) => {
                /* println!("[Type {}] Ext. Class B {}: {:?} {:?}",
                    ecb.message_type,
                    ecb.mmsi,
                    ecb.latitude.unwrap_or(0.0),
                    ecb.longitude.unwrap_or(0.0)
                ); */
            }
            AisMessage::DataLinkManagementMessage(dlm) => {
                /*  println!("[Type {}] DLM {}: Reservation {}",
                    dlm.message_type,
                    dlm.mmsi,
                    dlm.reservations.len()
                ); */
            }
            AisMessage::AidToNavigationReport(aton) => {
                 let (status_byte, page_id) = self.extract_aton_status(raw_sentence)? ;
                    // Parse status components for Page ID 7 (Most common operational status)
                   if page_id == 7 {
                        let (racon_status, light_status, health) = parse_aton_status(status_byte.reverse_bits());
                        println!(
                            "[Type {}] AtoN {}: {} ({:?})",
                            aton.message_type, aton.mmsi, aton.name, aton.aid_type
                        );
                        println!(
                            "  → Status: Page {} | RACON: {:?} | Light: {:?} | Off-position: {}",
                            page_id, racon_status, light_status, aton.off_position
                        );
                    

                    /* println!("[Type {}] AtoN {}: {} ({:?})",
                        aton.message_type,
                        aton.mmsi,
                        aton.name,
                        aton.aid_type
                    );
                    println!("  → Status: Page {} | RACON: {:?} | Light: {:?} | Off-position: {}",
                        page_id,
                        racon_status,
                        light_status,
                        aton.off_position
                    ); */
                }

                /* if (43.0..44.0).contains(&aton.latitude.unwrap_or(0 as f32)) &&
                   (16.0..17.0).contains(&aton.longitude.unwrap_or(0.0)) {
                    println!("Dalmatian AtoN: {}", aton.name);
                } */

                
               /*  println!("[Type {}] AtoN {}: {} ({:?})",
                    aton.message_type,
                    aton.mmsi,
                    aton.name,
                    aton.aid_type,);    */
                     
           }
            AisMessage::StaticDataReport(sdr) => {
                /* println!("[Type {}] Static Data {}: {:?}",
                    sdr.message_type,
                    sdr.mmsi,
                    sdr.message_part
                ); */
            }
            AisMessage::SafetyRelatedBroadcastMessage(srm) => {
                /* println!("[Type {}] Safety Message from {}: {}",
                    srm.message_type,
                    srm.mmsi,
                    srm.text
                ); */
            }
            AisMessage::BinaryAcknowledgeMessage(ba) => { /* Type 7  TODO*/ }

            AisMessage::BinaryBroadcastMessage(bbm) => {
                /* println!("[Type {}] Binary Broadcast {}: {} bytes",
                    bbm.message_type,
                    bbm.mmsi,
                    bbm.data.len()
                ); */
            }
            AisMessage::UtcDateResponse(udr) => {
                /*  println!("[Type {}] UTC Date- hour: {}{}:{} UTC",
                    udr.message_type,
                    udr.hour,
                    udr.minute.unwrap_or(0),
                    udr.second.unwrap_or(0),
                ); */
            }
            AisMessage::AssignmentModeCommand(amc) => {
                /*  println!("[Type {}] AMC for MMSI {}",
                    amc.message_type,
                    amc.mmsi
                ); */
            }
            // Add other message types as needed
            _ => println!("[Type s] Unhandled message format",),
        } */
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