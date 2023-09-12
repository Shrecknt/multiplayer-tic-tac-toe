use std::{net::SocketAddr, sync::Arc};

use common::common::{Board, BoardCell, C2SPlayPacket, Packet, S2CPlayPacket};
use tokio::{
    io::AsyncWriteExt,
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
    sync::Mutex,
};

use crate::server::GameState;

pub async fn handle_play(
    rstream: &mut OwnedReadHalf,
    wstream: Arc<Mutex<OwnedWriteHalf>>,
    board: Arc<Mutex<Board>>,
    addr: &SocketAddr,
    state: Arc<Mutex<GameState>>,
    team: usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Reached 'Play' state on {addr}");
    println!("Board state: {:?}", board.lock().await.cells);

    wstream
        .lock()
        .await
        .write_all(
            &S2CPlayPacket::UpdateCell {
                x: 2,
                y: 1,
                cell_type: BoardCell::X.to_usize(),
            }
            .serialize()?,
        )
        .await?;

    loop {
        let packet = C2SPlayPacket::deserialize(rstream).await?;
        if let C2SPlayPacket::UpdateCell { x, y, cell_type } = packet {
            if cell_type != team || state.lock().await.turn != team {
                wstream
                    .lock()
                    .await
                    .write_all(
                        &S2CPlayPacket::UpdateCell {
                            x,
                            y,
                            cell_type: board.lock().await.get(x, y).to_usize(),
                        }
                        .serialize()?,
                    )
                    .await?;
            }
        }
    }
}
