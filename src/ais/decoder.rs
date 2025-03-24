use super::msg21::{GeneralHealth, LightStatus, RaconStatus};
use ais::messages::AisMessageType;
use ais::{AisFragments, AisParser, messages::AisMessage};
use anyhow::Context;

use anyhow::Result;
use tokio::sync::mpsc::Sender;

#[derive(Debug)]
pub struct AisDecoder {
    pub parser: AisParser,
}

impl AisDecoder {
    pub fn new() -> Self {
        Self {
            parser: AisParser::new(),
        }
    }

    pub async fn process(&mut self, nmea_sentence: &str) -> anyhow::Result<String> {
        if let Ok((status_byte, page_id)) = self.extract_aton_status(nmea_sentence) {
            match page_id {
                // Handle RACON/Light Status
                0b111 => {
                    let (racon_status, light_status, health_status) =
                        parse_aton_status(status_byte);
                    Ok(format!(
                        "Page ID: {}, RACON: {:?}, Light: {:?}, Health: {:?}",
                        page_id, racon_status, light_status, health_status
                    ))
                }
                // Handle Mobile AtoN
                0b101 => {
                    let mobile_aton_status = parse_aton_status(status_byte);
                    Ok(format!(
                        "Page ID: {}, Mobile AtoN Status: {:?}",
                        page_id, mobile_aton_status
                    ))
                }
                // Handle Regional AtoN or other pages
                _ => Ok(format!(
                    "Page ID: {}, Status Byte: {:08b}",
                    page_id, status_byte
                )),
            }
        } else {
            Err(anyhow::anyhow!("Failed to extract AtoN status"))
        }
    }

    pub async fn handle_message(
        &self,
        msg: AisMessage,
        raw_sentence: &str,
        tx: Sender<AisMessage>,
    ) -> Result<()> {
        tx.send(msg).await?;

        Ok(())
    }

    fn payload_to_binary(&self, payload: &str) -> Option<String> {
        let mut binary = String::new();
        for c in payload.chars() {
            let ascii = c as u8;
            if !(48..=119).contains(&ascii) {
                // Valid AIS payload characters
                return None;
            }
            let bits = format!("{:06b}", ascii - 48);
            binary.push_str(&bits);
        }
        Some(binary)
    }
    /* fn extract_aton_status(&self, nmea_sentence: &str) -> Option<(u8, u8)> {
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
    } */
    fn extract_aton_status(&self, nmea_sentence: &str) -> anyhow::Result<(u8, u8)> {
        // Step 1: Validate NMEA structure
        let parts: Vec<&str> = nmea_sentence.split(',').collect();
        anyhow::ensure!(parts.len() >= 6, "Invalid NMEA format");

        // Step 2: Extract payload (5th field of NMEA sentence)
        let payload = parts[5];
        anyhow::ensure!(!payload.is_empty(), "Empty payload");

        // Step 3: Convert payload to binary representation
        let binary = self
            .payload_to_binary(payload)
            .context("Failed to convert payload to binary")?;

        // Step 4: Validate binary length (Message 21 requires at least 156 bits)
        anyhow::ensure!(
            binary.len() >= 156,
            format!("Payload too short ({} bits, expected >=156)", binary.len())
        );

        // Step 5: Extract status bits (148-155 inclusive)
        let start_index = binary.len() - 8;
        let status_bits = &binary[start_index..];
        let status_byte =
            u8::from_str_radix(status_bits, 2).context("Invalid binary status bits")?;
        //let status_byte = status_byte.reverse_bits(); //changed reverse
        // Step 6: Extract page ID (first 3 bits of the status byte)

        // let page_id = (status_byte >> 5) & 0b111;
        // Debugging Output
        /* println!("Binary Payload: {}", binary);
           println!("Status Bits [148-155]: {}", status_bits);
           println!("Status Byte (before reversal): {:08b}", status_byte);
        */
        // Reverse bits if necessary
        //let reverse_status_byte= status_byte.reverse_bits();
        //println!("Status Byte (after reversal): {:08b}", status_byte);

        let page_id = (status_byte >> 5) & 0b111;
        // println!("Page ID: {}", page_id);

        Ok((status_byte, page_id))
    }
}
fn parse_aton_status(status_byte: u8) -> (Option<RaconStatus>, Option<LightStatus>, GeneralHealth) {
    // Extract Page ID (Bits 8th, 7th, 6th)
    let page_id = (status_byte >> 5) & 0b111;

    match page_id {
        0b111 => {
            // Page ID = 7 (RACON/Light Status)
            let racon_bits = (status_byte >> 3) & 0b11; // Bits 5th and 4th
            let light_bits = (status_byte >> 1) & 0b11; // Bits 3rd and 2nd
            let health_bit = status_byte & 0b1; // Bit 1st

            (
                Some(match racon_bits {
                    0b00 => RaconStatus::NotFitted,
                    0b01 => RaconStatus::NotMonitored,
                    0b10 => RaconStatus::Operational,
                    0b11 => RaconStatus::Test,
                    _ => RaconStatus::Unknown,
                }),
                Some(match light_bits {
                    0b00 => LightStatus::NoLightOrNotMonitored,
                    0b01 => LightStatus::On,
                    0b10 => LightStatus::Off,
                    0b11 => LightStatus::FailOrReducedRange,
                    _ => LightStatus::Unknown,
                }),
                match health_bit {
                    0b0 => GeneralHealth::Good,
                    0b1 => GeneralHealth::Alarm,
                    _ => GeneralHealth::Unknown,
                },
            )
        }
        _ => (None, None, GeneralHealth::Unknown), // Other Page IDs not handled here
    }
}
