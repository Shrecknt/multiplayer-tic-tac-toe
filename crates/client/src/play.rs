use std::{io::Write, net::TcpStream, sync::Arc};

use common::common::{Board, BoardCell, C2SPlayPacket, Packet, S2CPlayPacket};
use tokio::sync::Mutex;

pub async fn handle_play(
    socket: &mut TcpStream,
    board: Arc<Mutex<Board>>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Current board state: {:?}", board.lock().await.cells);

    socket.write_all(
        C2SPlayPacket::UpdateCell {
            x: 1,
            y: 1,
            cell_type: BoardCell::X.to_u8(),
        }
        .serialize()?
        .as_slice(),
    )?;

    let mut packet: S2CPlayPacket;
    loop {
        packet = S2CPlayPacket::deserialize(socket)?;
        match packet {
            S2CPlayPacket::UpdateCell { x, y, cell_type } => {
                let mut board = board.lock().await;
                board.put(x, y, BoardCell::from_u8(cell_type)?)?;
                println!("x: {}, y: {}, cell_type: {}", x, y, cell_type);
            }
            _ => panic!("Unexpected packet {:?}", packet),
        }
    }
}
