use std::sync::Arc;

use tokio::{
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    sync::{mpsc, Mutex},
};

use common::common::{Board, DynamicRead, DynamicWrite};
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
    let (chat_stream_s, chat_stream_r) = mpsc::channel::<(String, String)>(256);

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
        if let Ok(stream) = TcpStream::connect(&addr).await {
            socket = stream;
            break;
        }
    }
    let (rstream, wstream) = socket.into_split();

    let (mut rstream, mut wstream) = into_boxed(rstream, wstream);

    const ENCRYPTION_ENABLED: bool = false;
    if ENCRYPTION_ENABLED {
        (rstream, wstream) = encryption::handle_encryption(rstream, wstream)
            .await
            .unwrap();
    }

    let wstream: Arc<Mutex<Box<DynamicWrite<'_>>>> = Arc::new(Mutex::new(wstream));

    login::handle_login(&mut rstream, wstream.clone(), board.clone())
        .await
        .unwrap();

    let play_wstream = wstream.clone();
    let play_board = board.clone();
    tokio::spawn(async move {
        play::handle_play(&mut rstream, play_wstream, play_board, chat_stream_s)
            .await
            .unwrap();
    });

    gui::main(wstream.clone(), board.clone(), chat_stream_r).unwrap();

    std::process::exit(0);
}

pub fn into_boxed(
    rstream: OwnedReadHalf,
    wstream: OwnedWriteHalf,
) -> (Box<DynamicRead<'static>>, Box<DynamicWrite<'static>>) {
    (Box::new(rstream), Box::new(wstream))
}
