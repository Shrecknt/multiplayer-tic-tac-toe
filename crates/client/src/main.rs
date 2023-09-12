use std::sync::Arc;

use tokio::{net::TcpStream, sync::Mutex};

use common::common::Board;
use gui;
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

    let board = Arc::new(parking_lot::Mutex::new(Board::new(0, 0)));

    let socket: TcpStream;
    loop {
        let addr = match gui::connection_screen() {
            Ok(addr) => match addr {
                Some(addr) => addr,
                None => {
                    println!("Clicked close button, exiting.");
                    std::process::exit(0);
                }
            },
            Err(_) => {
                continue;
            }
        };
        match TcpStream::connect(&addr).await {
            Ok(stream) => {
                socket = stream;
                break;
            }
            Err(_) => {}
        };
    }
    let (mut rstream, wstream) = socket.into_split();

    let wstream = Arc::new(Mutex::new(wstream));

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
