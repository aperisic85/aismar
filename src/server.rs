use tokio::net::TcpListener;
use crate::config::ServerConfig;
use crate::client::connection;

pub async fn run(config: ServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(&config.address).await?;
    println!("Server listening on {}", config.address);

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from: {}", addr);
        tokio::spawn(connection::handle_connection(socket));
    }
}
