use std::{
    io::Write,
    net::{SocketAddr, TcpStream},
    sync::Arc,
};

use common::common::{Board, BoardCell, Packet, S2CPlayPacket};
use tokio::sync::Mutex;

pub async fn handle_play(
    stream: &mut TcpStream,
    board: Arc<Mutex<Board>>,
    addr: &SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Reached 'Play' state on {addr}");
    println!("Board state: {:?}", board.lock().await.cells);

    stream.write_all(
        S2CPlayPacket::UpdateCell {
            x: 2,
            y: 1,
            cell_type: BoardCell::X.to_u8(),
        }
        .serialize()?
        .as_slice(),
    )?;

    Ok(())
}
