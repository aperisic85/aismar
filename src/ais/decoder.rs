use ais::{AisParser, messages::AisMessage, AisFragments};
use anyhow::Result;

pub struct AisDecoder {
    parser: AisParser,
}

impl AisDecoder {
    pub fn new() -> Self {
        Self {
            parser: AisParser::new(),
        }
    }

    pub async fn process(&mut self, nmea_sentence: &str) -> Result<()> {
        match self.parser.parse(nmea_sentence.as_bytes(), true) {
            Ok(fragments) => {
                // Pattern match on AisFragments variants
                if let AisFragments::Complete(sentence) = fragments {
                    if let Some(msg) = sentence.message {
                        self.handle_message(msg).await?;
                    }
                }
            }
            Err(e) => eprintln!("Decode error: {}", e),
        }
        Ok(())
    }

    // async fn handle_message(&self, msg: AisMessage) -> Result<()> {
    //     match msg {
    //         AisMessage::PositionReport(pos) => {
    //             println!("Vessel pos accuracy{}: {:?}", pos.mmsi, pos.position_accuracy);
    //         }
    //         AisMessage::AidToNavigationReport(aton) => {
    //             println!("AtoN {}: {}", aton.mmsi, aton.name);
    //         }
    //         AisMessage::DataLinkManagementMessage(dlm) => {
    //             println!("MMSI {} reserved slots", dlm.mmsi);
    //         }
    //         _ => println!("Unhandled message type: {:?}", msg),
    //     }
    //     Ok(())
    // }

    async fn handle_message(&self, msg: AisMessage) -> Result<()> {
        match msg {
            AisMessage::PositionReport(pos) => {
                println!("[Type {}] Vessel {}: {:?} {:?} | SOG: {} kt", 
                    pos.message_type,
                    pos.mmsi, 
                    pos.latitude.unwrap_or(0.0),
                    pos.longitude.unwrap_or(0.0),
                    pos.speed_over_ground.unwrap_or(0.0)
                );
            }
            AisMessage::BaseStationReport(bs) => {
                println!("[Type {}] Base Station {}: {:?} UTC", 
                    bs.message_type,
                    bs.mmsi,
                    bs.hour
                );
            }
            AisMessage::StaticAndVoyageRelatedData(sdv) => {
                println!("[Type {}] Vessel {}: {} → {}", 
                    sdv.message_type,
                    sdv.mmsi,
                    sdv.vessel_name,
                    sdv.destination
                );
            }
            AisMessage::StandardClassBPositionReport(scb) => {
                println!("[Type {}] Class B {}: {:?} {:?} COG: {}", 
                    scb.message_type,
                    scb.mmsi,
                    scb.latitude.unwrap_or(0.0),
                    scb.longitude.unwrap_or(0.0),
                    scb.course_over_ground.unwrap_or(0.0)
                );
            }
            AisMessage::ExtendedClassBPositionReport(ecb) => {
                println!("[Type {}] Ext. Class B {}: {:?} {:?}", 
                    ecb.message_type,
                    ecb.mmsi,
                    ecb.latitude.unwrap_or(0.0),
                    ecb.longitude.unwrap_or(0.0)
                );
            }
            AisMessage::DataLinkManagementMessage(dlm) => {
                println!("[Type {}] DLM {}: Reservation {}", 
                    dlm.message_type,
                    dlm.mmsi,
                    dlm.reservations.len()
                );
            }
            AisMessage::AidToNavigationReport(aton) => {
                println!("[Type {}] AtoN {}: {} ({:?})", 
                    aton.message_type,
                    aton.mmsi,
                    aton.name,
                    aton.aid_type
                );
            }
            AisMessage::StaticDataReport(sdr) => {
                println!("[Type {}] Static Data {}: {:?}", 
                    sdr.message_type,
                    sdr.mmsi,
                    sdr.message_part
                );
            }
            AisMessage::SafetyRelatedBroadcastMessage(srm) => {
                println!("[Type {}] Safety Message from {}: {}", 
                    srm.message_type,
                    srm.mmsi,
                    srm.text
                );
            }
            AisMessage::BinaryBroadcastMessage(bbm) => {
                println!("[Type {}] Binary Broadcast {}: {} bytes", 
                    bbm.message_type,
                    bbm.mmsi,
                    bbm.data.len()
                );
            }
            AisMessage::UtcDateResponse(udr) => {
                println!("[Type {}] UTC Date- hour: {}{}:{} UTC", 
                    udr.message_type,
                    udr.hour,
                    udr.minute.unwrap_or(0),
                    udr.second.unwrap_or(0),
                );
            }
            AisMessage::AssignmentModeCommand(amc) => {
                println!("[Type {}] AMC for MMSI {}", 
                    amc.message_type,
                    amc.mmsi
                );
            }
            // Add other message types as needed
            _ => println!("[Type s] Unhandled message format",),
        }
        Ok(())
    }
    
}
