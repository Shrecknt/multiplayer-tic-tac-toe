#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use common::common::Board;
use eframe::egui;
use tokio::{net::tcp::OwnedWriteHalf, sync::Mutex};

pub fn main(
    _wstream: Arc<Mutex<OwnedWriteHalf>>,
    board: Arc<parking_lot::Mutex<Board>>,
) -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };

    eframe::run_simple_native("Tic-Tac-Toe", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Multiplayer Tic-Tac-Toe");
            ui.label(format!("Board: {:?}", board.lock().cells));
        });
    })
}
