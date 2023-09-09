use std::{net::SocketAddr, sync::Arc};

use common::{
    common::{Board, C2SLoginPacket, Packet, S2CLoginPacket},
    VERSION_STRING,
};
use tokio::{
    io::AsyncWriteExt,
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
    sync::Mutex,
};

pub async fn handle_login(
    rstream: &mut OwnedReadHalf,
    wstream: Arc<Mutex<OwnedWriteHalf>>,
    board: Arc<Mutex<Board>>,
    addr: &SocketAddr,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Incoming connection from {}", addr);

    let mut packet = C2SLoginPacket::deserialize(rstream).await?;
    if let C2SLoginPacket::RequestVersion { client_version } = packet {
        if client_version != VERSION_STRING {
            println!("The client is not on the same version as the server. The client may decide to proceed with the connection.");
        }
        wstream
            .lock()
            .await
            .write_all(
                S2CLoginPacket::SendVersion {
                    server_version: VERSION_STRING.to_string(),
                }
                .serialize()?
                .as_slice(),
            )
            .await?;

        packet = C2SLoginPacket::deserialize(rstream).await?;
    }

    if let C2SLoginPacket::RequestBoard {} = packet {
    } else {
        return Err(format!("Unexpected packet {packet:?}").into());
    }

    let board = board.lock().await;
    wstream
        .lock()
        .await
        .write_all(
            S2CLoginPacket::UpdateBoardSize {
                width: board.width,
                height: board.height,
            }
            .serialize()?
            .as_slice(),
        )
        .await?;

    let mut y: usize = 0;
    for row in &board.cells {
        let mut x: usize = 0;
        for cell in row {
            wstream
                .lock()
                .await
                .write_all(
                    S2CLoginPacket::UpdateCell {
                        x,
                        y,
                        cell_type: cell.to_usize(),
                    }
                    .serialize()?
                    .as_slice(),
                )
                .await?;
            x += 1;
        }
        y += 1;
    }

    wstream
        .lock()
        .await
        .write_all(S2CLoginPacket::BoardSent {}.serialize()?.as_slice())
        .await?;

    Ok(())
}
