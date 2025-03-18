use crate::{ais::decoder, config::AisConfig};
use anyhow::Context;
use std::sync::Arc;
use tokio::io::AsyncBufReadExt;
use tokio::{io::BufReader, net::TcpStream, time};

pub struct AisConnection {
    stream: BufReader<TcpStream>,
    config: Arc<AisConfig>,
    decoder: decoder::AisDecoder,
}

impl AisConnection {
    pub fn new(stream: TcpStream, config: Arc<AisConfig>) -> Self {
        Self {
            stream: BufReader::new(stream),
            config,
            decoder: decoder::AisDecoder::new(),
        }
    }

    pub async fn handle(mut self) -> anyhow::Result<()> {
        let mut buffer = String::new();

        loop {
            buffer.clear();

            let read_result =
                time::timeout(self.config.read_timeout, self.stream.read_line(&mut buffer)).await;

            match read_result {
                Ok(Ok(0)) => break, // Clean disconnect
                Ok(Ok(_)) => {
                    let line = buffer.trim_end();
                    self.decoder.process(line).await?;
                }
                Ok(Err(e)) => return Err(e).context("Read error"),
                Err(_) => return Err(anyhow::anyhow!("Read timeout")),
            }
        }

        Ok(())
    }
}
