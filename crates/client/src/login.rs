use std::sync::Arc;

use common::{
    common::{Board, BoardCell, C2SLoginPacket, Packet, S2CLoginPacket},
    VERSION_STRING,
};
use tokio::{
    io::AsyncWriteExt,
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
    sync::Mutex,
};

const REQUEST_VERSION: bool = true;

pub async fn handle_login(
    rstream: &mut OwnedReadHalf,
    wstream: Arc<Mutex<OwnedWriteHalf>>,
    board: Arc<parking_lot::Mutex<Board>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if REQUEST_VERSION {
        wstream
            .lock()
            .await
            .write_all(
                &C2SLoginPacket::RequestVersion {
                    client_version: VERSION_STRING.to_string(),
                }
                .serialize()?,
            )
            .await?;

        let version_packet = S2CLoginPacket::deserialize(rstream).await?;
        if let S2CLoginPacket::SendVersion { server_version } = version_packet {
            if server_version == VERSION_STRING {
                println!("The client and the server are on the same version.");
            } else {
                println!(
                    "The client and the server are not on the same version, proceeding anyways."
                );
            }
        } else {
            return Err(format!("Expected version packet, got {:?}", version_packet).into());
        }
    }

    wstream
        .lock()
        .await
        .write_all(&C2SLoginPacket::RequestBoard {}.serialize()?)
        .await?;

    let mut recieve_board_packet: S2CLoginPacket;
    loop {
        recieve_board_packet = S2CLoginPacket::deserialize(rstream).await?;
        match recieve_board_packet {
            S2CLoginPacket::UpdateCell { x, y, cell_type } => {
                let mut board = board.lock();
                board.put(x, y, BoardCell::from_usize(cell_type)?)?;
            }
            S2CLoginPacket::UpdateBoardSize { width, height } => {
                let mut board = board.lock();
                board.width = width;
                board.height = height;
            }
            S2CLoginPacket::BoardSent {} => {
                return Ok(());
            }
            _ => return Err(format!("Unexpected packet {:?}", recieve_board_packet).into()),
        }
    }
}
