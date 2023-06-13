mod client_connection;
mod maze;

use client_connection::ClientConnection;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> client_connection::Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("Starting on {}...", addr);

    // accept connections and process them serially
    loop {
        let (socket, _) = listener.accept().await?;
        println!("Got a new connection");
        let conn = ClientConnection::new(socket);
        tokio::spawn(conn.handle());
    }
}
