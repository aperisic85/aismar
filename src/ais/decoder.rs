use ais::AisParser;
use ais::*;
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
                if let Some(msg) = fragments.into_message() {
                    self.handle_message(msg).await?;
                }
            }
            Err(e) => eprintln!("Decode error: {}", e),
        }
        Ok(())
    }

    async fn handle_message(&self, msg: ais::messages::AisMessage) -> Result<()> {
        match msg {
            ais::messages::AisMessage::PositionReport(pos) => {
                println!("Vessel {}: {:?}", pos.mmsi, pos.position);
            }
            ais::messages::AisMessage::AidToNavigationReport(aton) => {
                println!("AtoN {}: {}", aton.mmsi, aton.name);
            }
            _ => {} // Handle other message types
        }
        Ok(())
    }
}
