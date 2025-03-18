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

    async fn handle_message(&self, msg: AisMessage) -> Result<()> {
        match msg {
            AisMessage::PositionReport(pos) => {
                println!("Vessel pos accuracy{}: {:?}", pos.mmsi, pos.position_accuracy);
            }
            AisMessage::AidToNavigationReport(aton) => {
                println!("AtoN {}: {}", aton.mmsi, aton.name);
            }
            AisMessage::DataLinkManagementMessage(dlm) => {
                println!("MMSI {} reserved slots", dlm.mmsi);
            }
            _ => println!("Unhandled message type: {:?}", msg),
        }
        Ok(())
    }
}
