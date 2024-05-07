use std::net::SocketAddr;

use mcpi::connection::ServerConnection;

#[tokio::main]
pub async fn main() {
    let args: Vec<_> = std::env::args().collect();
    let addr = match args.get(1) {
        Some(addr) => addr.as_ref(),
        None => "raspberrypi.local:4711",
    };

    let connection = ServerConnection::new(addr).await.unwrap();
}
