#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use common::common::Board;
use eframe::{egui, IconData};
use tokio::{net::tcp::OwnedWriteHalf, sync::Mutex};

pub fn main(
    _wstream: Arc<Mutex<OwnedWriteHalf>>,
    board: Arc<parking_lot::Mutex<Board>>,
) -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        resizable: false,
        icon_data: Some(
            IconData::try_from_png_bytes(include_bytes!("../assets/icon.png")).unwrap(),
        ),
        ..Default::default()
    };

    let mut assets = Assets::unloaded();

    eframe::run_simple_native("Tic-Tac-Toe", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Multiplayer Tic-Tac-Toe");
            ui.label(format!("Board: {:?}", board.lock().cells));

            let icon_texture = assets.icon_texture(ui);
            ui.image(icon_texture, egui::vec2(64.0, 64.0));

            let x_texture = assets.x_texture(ui);
            ui.image(x_texture, egui::vec2(64.0, 64.0));

            let o_texture = assets.o_texture(ui);
            ui.image(o_texture, egui::vec2(64.0, 64.0));
        });
    })
}

pub fn connection_screen() -> Result<Option<String>, eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        resizable: false,
        icon_data: Some(
            IconData::try_from_png_bytes(include_bytes!("../assets/icon.png")).unwrap(),
        ),
        ..Default::default()
    };

    let mut addr = "localhost:21552".to_owned();
    let final_addr = Arc::new(parking_lot::Mutex::new("".to_string()));
    let final_addr_clone = final_addr.clone();
    let final_exit = Arc::new(parking_lot::Mutex::new(false));
    let final_exit_clone = final_exit.clone();

    eframe::run_simple_native("Tic-Tac-Toe", options, move |ctx, frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Multiplayer Tic-Tac-Toe");

            ui.horizontal(|ui| {
                let addr_label = ui.label("Host address: ");
                ui.text_edit_singleline(&mut addr)
                    .labelled_by(addr_label.id);
            });

            ui.horizontal(|ui| {
                if ui.button("connect").clicked() {
                    let mut addr_arc = final_addr_clone.lock();
                    *addr_arc = addr.clone();
                    let mut exit_arc = final_exit_clone.lock();
                    *exit_arc = false;
                    frame.close();
                }
                if ui.button("exit").clicked() {
                    println!("exit = true");
                    let mut exit_arc = final_exit_clone.lock();
                    *exit_arc = true;
                    frame.close();
                }
            });
        });
    })?;

    let exit = *final_exit.lock();
    if exit {
        Ok(None)
    } else {
        let addr = final_addr.lock();
        Ok(Some(addr.clone()))
    }
}

struct Assets {
    icon_texture: Option<egui::TextureHandle>,
    x_texture: Option<egui::TextureHandle>,
    o_texture: Option<egui::TextureHandle>,
}

impl Assets {
    pub fn unloaded() -> Self {
        Self {
            icon_texture: None,
            x_texture: None,
            o_texture: None,
        }
    }
    fn icon_texture(&mut self, ui: &egui::Ui) -> &egui::TextureHandle {
        self.icon_texture.get_or_insert_with(|| {
            ui.ctx().load_texture(
                "icon",
                load_image_from_memory(include_bytes!("../assets/icon.png")).unwrap(),
                Default::default(),
            )
        })
    }
    fn x_texture(&mut self, ui: &egui::Ui) -> &egui::TextureHandle {
        self.x_texture.get_or_insert_with(|| {
            ui.ctx().load_texture(
                "x",
                load_image_from_memory(include_bytes!("../assets/x.png")).unwrap(),
                Default::default(),
            )
        })
    }
    fn o_texture(&mut self, ui: &egui::Ui) -> &egui::TextureHandle {
        self.o_texture.get_or_insert_with(|| {
            ui.ctx().load_texture(
                "o",
                load_image_from_memory(include_bytes!("../assets/o.png")).unwrap(),
                Default::default(),
            )
        })
    }
}

fn load_image_from_memory(image_data: &[u8]) -> Result<egui::ColorImage, image::ImageError> {
    let image = image::load_from_memory(image_data)?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(egui::ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    ))
}
