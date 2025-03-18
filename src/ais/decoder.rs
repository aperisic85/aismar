use ais::{AisParser, messages::AisMessage, AisFragments};
use super::msg21::{RaconStatus, LightStatus};

use anyhow::Result;

#[derive(Debug)]
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
                        self.handle_message(msg, nmea_sentence).await?;
                    }
                }
            }
            Err(e) => eprintln!("Decode error: {}", e),
        }
        Ok(())
    }

  

    async fn handle_message(&self, msg: AisMessage,raw_sentence: &str) -> Result<()> {
        match msg {
            AisMessage::PositionReport(pos) => {
                /* println!("[Type {}] Vessel {}: {:?} {:?} | SOG: {} kt", 
                    pos.message_type,
                    pos.mmsi, 
                    pos.latitude.unwrap_or(0.0),
                    pos.longitude.unwrap_or(0.0),
                    pos.speed_over_ground.unwrap_or(0.0)
                ); */
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
                
                if let Some((status_byte, page_id)) = self.extract_aton_status(raw_sentence) {
                    // Parse status components for Page ID 7 (Most common operational status)
                    if page_id == 7 {
                        let (racon_status, light_status) = parse_aton_status(status_byte, page_id);
                        println!("[Type {}] AtoN {}: {} ({:?})",
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
                        );
                    }
                    
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
            
               
                if (43.0..44.0).contains(&aton.latitude.decimal_degrees()) &&
                   (16.0..17.0).contains(&aton.longitude.decimal_degrees()) {
                    println!("Dalmatian AtoN: {}", aton.name);
                }
                
                
             
                println!("[Type {}] AtoN {}: {} ({:?})", 
                    aton.message_type,
                    aton.mmsi,
                    aton.name,
                    aton.aid_type,);
                    
               

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
            AisMessage::BinaryAddressedMessage(bam) => { /* Type 6 TODO*/ }
            AisMessage::BinaryAcknowledge(ba) => { /* Type 7  TODO*/ }
            
            AisMessage::BinaryBroadcastMessage(bbm) => {
                /* println!("[Type {}] Binary Broadcast {}: {} bytes", 
                    bbm.message_type,
                    bbm.mmsi,
                    bbm.data.len()
                ); */
            }
            AisMessage::StandardSarAircraftReport(sar) => { /* Type 9 TODO */ }
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
        }
        Ok(())
    }

    
    fn payload_to_binary(&self, payload: &str) -> Option<String> {
        let mut binary = String::new();
        for c in payload.chars() {
            let ascii = c as u8;
            if !(48..=119).contains(&ascii) {  // Valid AIS payload characters
                return None;
            }
            let bits = format!("{:06b}", ascii - 48);
            binary.push_str(&bits);
        }
        Some(binary)
    }
    fn extract_aton_status(&self, nmea_sentence: &str) -> Option<(u8, u8)> {
        let payload = nmea_sentence.split(',').nth(5)?;
        let binary = self.payload_to_binary(payload)?;
        
        if binary.len() >= 156 {
            let status_bits = &binary[148..156];
            let status_byte = u8::from_str_radix(status_bits, 2).ok()?;
            let page_id = (status_byte >> 5) & 0b111;  // Extract first 3 bits
            
            Some((status_byte, page_id))
        } else {
            None
        }
    }
    
}
pub fn parse_aton_status(status_byte: u8, page_id: u8) -> (RaconStatus, LightStatus) {
    match page_id {
        7 => {  // IALA Page 7: Operational Status
            let racon_bits = (status_byte >> 3) & 0b11;
            let light_bits = status_byte & 0b11;
            
            let racon_status = match racon_bits {
                0b00 => RaconStatus::NotFitted,
                0b01 => RaconStatus::NotMonitored,
                0b10 => RaconStatus::Operational,
                0b11 => RaconStatus::Test,
                _ => unreachable!(),
            };

            let light_status = match light_bits {
                0b00 => LightStatus::None,
                0b01 => LightStatus::On,
                0b10 => LightStatus::Reserved,
                0b11 => LightStatus::Off,
                _ => unreachable!(),
            };

            (racon_status, light_status)
        }
        _ => (RaconStatus::Unknown, LightStatus::Unknown),
    }
}

