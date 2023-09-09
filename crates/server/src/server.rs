use std::{collections::HashMap, sync::Arc};

use common::common::Board;
use tokio::{net::TcpListener, sync::Mutex};

use crate::{login, play};

pub struct GameState {
    pub turn: usize,
}

pub async fn start_server(hostname: &str) -> Result<(), Box<dyn std::error::Error>> {
    let socket = TcpListener::bind(hostname).await?;

    let board = Arc::new(Mutex::new(Board::new(3, 3)));
    board
        .lock()
        .await
        .put(1, 1, common::common::BoardCell::X)
        .unwrap();

    let state = Arc::new(Mutex::new(GameState { turn: 0 }));

    let mut user_map = HashMap::new();

    loop {
        let board = board.clone();
        let state = state.clone();

        let (stream, addr) = socket.accept().await?;
        let (mut rstream, wstream) = stream.into_split();

        let wstream = Arc::new(Mutex::new(wstream));

        user_map.insert(addr, wstream.clone());

        tokio::spawn(async move {
            login::handle_login(&mut rstream, wstream.clone(), board.clone(), &addr)
                .await
                .unwrap();

            let team = 0;
            play::handle_play(
                &mut rstream,
                wstream.clone(),
                board.clone(),
                &addr,
                state,
                team,
            )
            .await
            .unwrap();
        });
    }
}
