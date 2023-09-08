use std::{
    io::Write,
    net::{SocketAddr, TcpStream},
    sync::Arc,
};

use common::common::{Board, C2SLoginPacket, Packet, S2CLoginPacket};
use tokio::sync::Mutex;

pub async fn handle_login(
    stream: &mut TcpStream,
    board: Arc<Mutex<Board>>,
    addr: &SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Incoming connection from {}", addr);

    let packet = C2SLoginPacket::deserialize(stream)?;
    if let C2SLoginPacket::RequestBoard {} = packet {
    } else {
        return Err(format!("Unexpected packet {packet:?}").into());
    }

    let board = board.lock().await;
    stream.write_all(
        S2CLoginPacket::UpdateBoardSize {
            width: board.width,
            height: board.height,
        }
        .serialize()?
        .as_slice(),
    )?;

    let mut y: usize = 0;
    for row in &board.cells {
        let mut x: usize = 0;
        for cell in row {
            stream.write_all(
                S2CLoginPacket::UpdateCell {
                    x,
                    y,
                    cell_type: cell.to_u8(),
                }
                .serialize()?
                .as_slice(),
            )?;
            x += 1;
        }
        y += 1;
    }

    stream.write_all(S2CLoginPacket::BoardSent {}.serialize()?.as_slice())?;

    Ok(())
}
