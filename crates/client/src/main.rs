use tokio::net::TcpStream;

use server::server::start_server;

const LOCAL_SERVER: bool = false;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");

    if LOCAL_SERVER {
        tokio::spawn(async move {
            start_server("localhost:21552").await.unwrap();
        });
    }

    let socket = TcpStream::connect("localhost:21552").await?;
    let (mut socketr, mut socketw) = socket.into_split();

    drop(socketr);
    drop(socketw);

    Ok(())
}
