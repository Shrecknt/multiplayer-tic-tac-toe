use std::{io, io::Write, sync::Arc};

use common::common::{Board, BoardCell, C2SPlayPacket, Packet, S2CPlayPacket};
use tokio::{
    io::AsyncWriteExt,
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
    sync::Mutex,
};

pub async fn handle_play(
    rstream: &mut OwnedReadHalf,
    wstream: Arc<Mutex<OwnedWriteHalf>>,
    board: Arc<parking_lot::Mutex<Board>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // println!("Current board state: {:?}", board.lock().await.cells);
    let x: usize = python_input("X: ").unwrap().trim().parse().unwrap();
    let y: usize = python_input("Y: ").unwrap().trim().parse().unwrap();
    if board.lock().occupied(x, y) {
        println!("Occupied!")
    } else {
        wstream
            .lock()
            .await
            .write_all(
                &C2SPlayPacket::UpdateCell {
                    x,
                    y,
                    cell_type: BoardCell::X.to_usize(),
                }
                .serialize()?,
            )
            .await?;
        board
            .lock()
            .put(x, y, BoardCell::X)
            .expect("TODO: panic message");
    }
    let mut packet: S2CPlayPacket;
    loop {
        packet = S2CPlayPacket::deserialize(rstream).await?;
        match packet {
            S2CPlayPacket::UpdateCell { x, y, cell_type } => {
                let mut board = board.lock();
                board.put(x, y, BoardCell::from_usize(cell_type)?)?;
                println!("x: {}, y: {}, cell_type: {}", x, y, cell_type);
            }
            _ => panic!("Unexpected packet {:?}", packet),
        }
    }
}
fn python_input(input: &str) -> Option<String> {
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
