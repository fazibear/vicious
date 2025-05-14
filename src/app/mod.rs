use std::time::Duration;

use anyhow::Result;
use eframe::{
    egui,
    egui::{CentralPanel, Context, Slider},
    Frame,
};

use crate::{player::Player, sound::Sound};

pub struct App {
    sound: Sound,
    player: Player,
}

impl Default for App {
    fn default() -> Self {
        Self::new().expect("create new app")
    }
}

impl App {
    pub fn new() -> Result<Self> {
        let filename = std::env::args().nth(1).unwrap_or("".to_string());
        let path = std::path::Path::new(&filename);
        let data = std::fs::read(path)?;

        let sound = Sound::new()?;
        let mut player = Player::new(&data)?;

        player.play();

        Ok(Self { sound, player })
    }

    pub fn step(&mut self) {
        if !self.player.playing {
            return;
        }

        if let Some(data) = self.player.data() {
            self.sound.write_blocking(&data[..]);
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        self.step();
        ctx.request_repaint();
        egui::TopBottomPanel::top("Player").show(ctx, |ui| {
            //PlayerComponent::add(self, ui);
            //ScopeComponent::add(self, ui);
        });

        egui::TopBottomPanel::bottom("Footer").show(ctx, |ui| {
            //Footer::add(self, ui);
        });

        egui::CentralPanel::default().show(ctx, |_ui| {
            egui::SidePanel::left("Library Window")
                .default_width(350.0)
                .show(ctx, |ui| {
                    //LibraryComponent::add(self, ui);
                });
        });

        egui::CentralPanel::default().show(ctx, |_ui| {
            egui::CentralPanel::default().show(ctx, |ui| {});
        });
    }
}
