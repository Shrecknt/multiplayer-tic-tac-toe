use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use common::common::{Board, Packet, S2CLoginPacket, S2CPlayPacket};
use tokio::{
    io::AsyncWriteExt,
    net::{tcp::OwnedWriteHalf, TcpListener},
    sync::Mutex,
};

use crate::{login, play};

pub struct GameState {
    pub turn: usize,
    pub user_map: Arc<Mutex<HashMap<SocketAddr, Arc<Mutex<OwnedWriteHalf>>>>>,
}

pub async fn start_server(hostname: &str) -> Result<(), Box<dyn std::error::Error>> {
    let socket = TcpListener::bind(hostname).await?;

    let board = Arc::new(Mutex::new(Board::new(3, 3)));
    board
        .lock()
        .await
        .put(1, 1, common::common::BoardCell::X)
        .unwrap();

    let user_map = Arc::new(Mutex::new(HashMap::new()));
    let state = Arc::new(Mutex::new(GameState {
        turn: 0,
        user_map: user_map.clone(),
    }));

    loop {
        let board = board.clone();
        let user_map = user_map.clone();
        let state = state.clone();

        let (stream, addr) = socket.accept().await?;
        let (mut rstream, wstream) = stream.into_split();

        let wstream = Arc::new(Mutex::new(wstream));

        user_map.lock().await.insert(addr, wstream.clone());

        tokio::spawn(async move {
            match login::handle_login(&mut rstream, wstream.clone(), board.clone(), &addr).await {
                Ok(_) => {}
                Err(err) => {
                    user_map.lock().await.remove(&addr);
                    let _ = wstream
                        .lock()
                        .await
                        .write_all(
                            &S2CLoginPacket::Kick {
                                reason: err.to_string(),
                            }
                            .serialize()
                            .unwrap(),
                        )
                        .await;
                    return;
                }
            }

            let team = 0;
            match play::handle_play(
                &mut rstream,
                wstream.clone(),
                board.clone(),
                &addr,
                state,
                team,
            )
            .await
            {
                Ok(_) => {
                    user_map.lock().await.remove(&addr);
                }
                Err(err) => {
                    user_map.lock().await.remove(&addr);
                    let _ = wstream
                        .lock()
                        .await
                        .write_all(
                            &S2CPlayPacket::Kick {
                                reason: err.to_string(),
                            }
                            .serialize()
                            .unwrap(),
                        )
                        .await;
                }
            }
        });
    }
}
