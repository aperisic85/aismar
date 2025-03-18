use std::time::Duration;

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