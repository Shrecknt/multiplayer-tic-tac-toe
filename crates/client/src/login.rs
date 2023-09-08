use std::{io::Write, net::TcpStream, sync::Arc};

use common::common::{Board, BoardCell, C2SLoginPacket, Packet, S2CLoginPacket};
use tokio::sync::Mutex;

pub async fn handle_login(
    socket: &mut TcpStream,
    board: Arc<Mutex<Board>>,
) -> Result<(), Box<dyn std::error::Error>> {
    socket.write_all(C2SLoginPacket::RequestBoard {}.serialize()?.as_slice())?;

    let mut recieve_board_packet: S2CLoginPacket;
    loop {
        recieve_board_packet = S2CLoginPacket::deserialize(socket)?;
        match recieve_board_packet {
            S2CLoginPacket::UpdateCell { x, y, cell_type } => {
                let mut board = board.lock().await;
                board.put(x, y, BoardCell::from_u8(cell_type)?)?;
            }
            S2CLoginPacket::UpdateBoardSize { width, height } => {
                let mut board = board.lock().await;
                board.width = width;
                board.height = height;
            }
            S2CLoginPacket::BoardSent {} => {
                return Ok(());
            }
            _ => panic!("Unexpected packet {:?}", recieve_board_packet),
        }
    }
}
