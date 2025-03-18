use std::time::Duration;

#[derive(Clone, Debug)]
pub struct AisConfig {
    pub endpoint: String,
    pub max_reconnect_attempts: usize,
    pub reconnect_delay: Duration,
    pub read_timeout: Duration,
}

impl Default for AisConfig {
    fn default() -> Self {
        Self {
            endpoint: "192.168.51.121:4712".into(),
            max_reconnect_attempts: 5,
            reconnect_delay: Duration::from_secs(5),
            read_timeout: Duration::from_secs(30),
        }
    }
}
