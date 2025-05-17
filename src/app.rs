use anyhow::Result;
use eframe::{
    egui::{self, Context, ScrollArea},
    Frame,
};

use crate::{files::Files, playback::Playback, player::Player};

pub struct App {
    playback: Playback,
    player: Player,
    volume: f32,
    status: String,
    files: Files,
}

impl Default for App {
    fn default() -> Self {
        Self::new().expect("create new app")
    }
}

impl App {
    pub fn new() -> Result<Self> {
        let playback = Playback::new()?;
        let player = Player::new();

        let volume = 1.0;
        let status = "Started...".to_owned();

        let files = Files::new();

        Ok(Self {
            playback,
            player,
            volume,
            status,
            files,
        })
    }

    pub fn load(&mut self, filename: &str) -> Result<()> {
        let path = std::path::Path::new(&filename);
        let data = std::fs::read(path)?;
        self.player.load_data(&data)?;
        self.player.play();
        Ok(())
    }

    pub fn step(&mut self) {
        if !self.player.playing {
            return;
        }

        if let Some(data) = self.player.data() {
            self.playback.write_blocking(&data[..]);
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        ctx.request_repaint();
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            egui::SidePanel::right("right_panel").show_inside(ui, |ui| {
                egui::Grid::new("song_addteses")
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label("Data length");
                        ui.label("0x0000");
                        ui.end_row();
                        ui.label("Init address:");
                        ui.label("0x0000");
                        ui.end_row();
                        ui.label("Play address:");
                        ui.label("0x0000");
                        ui.end_row();
                        ui.label("Load address:");
                        ui.label("0x0000");
                        ui.end_row();
                    });
            });
            egui::Grid::new("song_info").num_columns(2).show(ui, |ui| {
                ui.label("Song:");
                ui.label("terefere tralalala bum");
                ui.end_row();
                ui.label("Author:");
                ui.label("the bytels");
                ui.end_row();
                ui.label("Released:");
                ui.label("1998");
                ui.end_row();
                ui.label("Number of songs:");
                ui.label("10");
                ui.end_row();
            });
        });
        egui::TopBottomPanel::top("topxxx")
            .min_height(40.0)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    let prev_btn = ui.button("|◀");
                    let prev_btn = ui.button("◀◀");
                    let play_btn = ui.button("▶");
                    let pause_btn = ui.button("⏸");
                    let stop_btn = ui.button("■");
                    let next_btn = ui.button("▶▶");
                    let next_btn = ui.button("▶|");
                    let volume_slider = ui.add(
                        eframe::egui::Slider::new(&mut self.volume, (0.0 as f32)..=(1.2 as f32))
                            .logarithmic(false)
                            .show_value(false)
                            .step_by(0.01),
                    );
                });
            });
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.label(&self.status);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                self.files.show(ui);
            });
        });
    }
}
