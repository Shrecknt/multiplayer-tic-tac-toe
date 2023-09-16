use std::sync::Arc;

use tokio::{io::AsyncWrite, net::TcpStream, sync::Mutex};

use common::common::Board;
use gui;
use server::server::start_server;

pub mod encryption;
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

    let board = Arc::new(parking_lot::Mutex::new(Board::new(0, 0)));

    let socket = TcpStream::connect("localhost:21552").await?;
    let (rstream, wstream) = socket.into_split();

    let (mut rstream, wstream) = encryption::handle_encryption(rstream, wstream)
        .await
        .unwrap();

    let wstream: Arc<Mutex<Box<dyn AsyncWrite + Unpin + Send + Sync>>> =
        Arc::new(Mutex::new(wstream));

    login::handle_login(&mut rstream, wstream.clone(), board.clone())
        .await
        .unwrap();

    let play_wstream = wstream.clone();
    let play_board = board.clone();
    tokio::spawn(async move {
        play::handle_play(&mut rstream, play_wstream, play_board)
            .await
            .unwrap();
    });

    gui::main(wstream.clone(), board.clone()).unwrap();

    std::process::exit(0);
}
