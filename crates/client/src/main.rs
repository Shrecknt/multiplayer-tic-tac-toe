use std::sync::Arc;

use tokio::{net::TcpStream, sync::Mutex};

use common::common::Board;
use server::server::start_server;

pub mod login;
pub mod play;

const LOCAL_SERVER: bool = false;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if LOCAL_SERVER {
        tokio::spawn(async move {
            start_server("localhost:21552").await.unwrap();
        });
    }

    let board = Arc::new(Mutex::new(Board::new(0, 0)));

    let socket = TcpStream::connect("localhost:21552").await?;
    let mut socket = socket.into_std()?;
    socket.set_nonblocking(false)?;

    login::handle_login(&mut socket, board.clone()).await?;
    play::handle_play(&mut socket, board.clone()).await?;

    Ok(())
}
