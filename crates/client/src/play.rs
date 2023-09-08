use std::{io, io::Write, net::TcpStream, sync::Arc};

use common::common::{Board, BoardCell, C2SPlayPacket, Packet, S2CPlayPacket};
use tokio::sync::Mutex;

pub async fn handle_play(
    socket: &mut TcpStream,
    board: Arc<Mutex<Board>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // println!("Current board state: {:?}", board.lock().await.cells);
    let x: usize  = python_input("X: ").unwrap().trim().parse().unwrap();
    let y: usize = python_input("Y: ").unwrap().trim().parse().unwrap();
    if board.lock().await.occupied(x, y) {
        println!("Occupied!")
    } else {
        socket.write_all(
            C2SPlayPacket::UpdateCell {
                x,
                y,
                cell_type: BoardCell::X.to_u8(),
            }
                .serialize()?
                .as_slice(),
        )?;
        board.lock().await.put(x, y, BoardCell::X).expect("TODO: panic message");
    }
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
fn python_input(input: &str) -> Option<String>{
    print!("{input}");
    io::stdout().flush().unwrap();
    read_line()
}
fn read_line() -> Option<String> {
    let mut input = String::new();
    let unwrap = io::stdin().read_line(&mut input);
    if unwrap.is_ok() {
        return Some(input);
    }
    None
}
