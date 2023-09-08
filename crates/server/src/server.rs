use std::sync::Arc;

use common::common::Board;
use tokio::{net::TcpListener, sync::Mutex};

use crate::{login, play};

pub async fn start_server(hostname: &str) -> Result<(), Box<dyn std::error::Error>> {
    let socket = TcpListener::bind(hostname).await?;

    let board = Arc::new(Mutex::new(Board::new(3, 3)));
    board.lock().await.put(1, 1, common::common::BoardCell::X)?;

    loop {
        let board = board.clone();
        let (stream, addr) = socket.accept().await?;
        let mut stream = stream.into_std()?;
        stream.set_nonblocking(false)?;

        tokio::spawn(async move {
            login::handle_login(&mut stream, board.clone(), &addr)
                .await
                .unwrap();

            play::handle_play(&mut stream, board.clone(), &addr)
                .await
                .unwrap();
        });
    }
}
