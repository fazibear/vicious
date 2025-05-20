use crate::{files, output::Output, sid_player::SidPlayer};
use anyhow::Result;
use eframe::{
    egui::{self, mutex::Mutex, CollapsingHeader, Context, ScrollArea, Ui},
    Frame,
};
use rb::{SpscRb, RB};
use serde_json::Value;
use sid_file::SidFile;
use std::sync::Arc;

pub struct App {
    sid_file: Option<SidFile>,
    pub sid_player: Arc<Mutex<SidPlayer>>,
    status: String,
    json: Value,
    _buffer: SpscRb<i16>,
    _output: Output,
}

impl Default for App {
    fn default() -> Self {
        Self::new().expect("create new app")
    }
}

impl App {
    pub fn new() -> Result<Self> {
        let sid_file = None;
        let buffer: SpscRb<i16> = SpscRb::new(44100 * 2);
        let output = Output::new(buffer.consumer())?;
        let sid_player = SidPlayer::new(buffer.producer(), output.sample_rate());
        let status = "Started...".to_owned();
        let json = files::files();

        let sid_player = Arc::new(Mutex::new(sid_player));

        let sid_player_thread = sid_player.clone();
        std::thread::spawn(move || loop {
            sid_player_thread.lock().step();
        });

        Ok(Self {
            sid_file,
            sid_player,
            status,
            json,
            _buffer: buffer,
            _output: output,
        })
    }

    pub fn load(&mut self, filename: &str) -> Result<()> {
        let data = files::open(filename)?;
        let sid_file = SidFile::parse(&data)?;

        self.sid_player.lock().load_data(
            &sid_file.data,
            sid_file.real_load_address,
            sid_file.init_address,
            sid_file.play_address,
            sid_file.songs,
            sid_file.start_song,
        );
        self.sid_file = Some(sid_file);
        self.sid_player.lock().play();
        Ok(())
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        ctx.request_repaint();
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            egui::SidePanel::right("right_panel").show_inside(ui, |ui| {
                let length = if let Some(file) = &self.sid_file {
                    &format!("0x{:04x}", file.data.len())
                } else {
                    ""
                };
                let init_address = if let Some(file) = &self.sid_file {
                    &format!("0x{:04x}", file.init_address)
                } else {
                    ""
                };
                let play_address = if let Some(file) = &self.sid_file {
                    &format!("0x{:04x}", file.play_address)
                } else {
                    ""
                };
                let load_address = if let Some(file) = &self.sid_file {
                    &format!("0x{:04x}", file.load_address)
                } else {
                    ""
                };
                egui::Grid::new("song_addteses")
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label("Data length");
                        ui.label(length);
                        ui.end_row();
                        ui.label("Init address:");
                        ui.label(init_address);
                        ui.end_row();
                        ui.label("Play address:");
                        ui.label(play_address);
                        ui.end_row();
                        ui.label("Load address:");
                        ui.label(load_address);
                        ui.end_row();
                    });
            });
            egui::Grid::new("song_info").num_columns(2).show(ui, |ui| {
                let song = if let Some(file) = &self.sid_file {
                    &file.name
                } else {
                    ""
                };
                let author = if let Some(file) = &self.sid_file {
                    &file.author
                } else {
                    ""
                };
                let released = if let Some(file) = &self.sid_file {
                    &file.released
                } else {
                    ""
                };
                let songs = if let Some(file) = &self.sid_file {
                    &format!("{}", &file.songs)
                } else {
                    ""
                };
                ui.label("Song:");
                ui.label(song);
                ui.end_row();
                ui.label("Author:");
                ui.label(author);
                ui.end_row();
                ui.label("Released:");
                ui.label(released);
                ui.end_row();
                ui.label("Number of songs:");
                ui.label(songs);
                ui.end_row();
            });
        });
        egui::TopBottomPanel::top("topxxx")
            .min_height(40.0)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    if ui.button("|◀").clicked() {};
                    if ui.button("◀◀").clicked() {};
                    if ui.button("▶").clicked() {};
                    if ui.button("⏸").clicked() {};
                    if ui.button("■").clicked() {};
                    if ui.button("▶▶").clicked() {};
                    if ui.button("▶|").clicked() {};
                    // let volume_slider = ui.add(
                    //     eframe::egui::Slider::new(&mut self.volume, (0.0 as f32)..=(1.2 as f32))
                    //         .logarithmic(false)
                    //         .show_value(false)
                    //         .step_by(0.01),
                    // );
                });
            });
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.label(&self.status);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                self.show(ui);
            });
        });
    }
}

impl App {
    pub fn add_dir(&mut self, ui: &mut Ui, value: &Value) {
        if let Some(vv) = value.get("type") {
            match vv.as_str() {
                Some("directory") => {
                    let zero = Value::from(0);
                    let empty = Vec::new();

                    let contents = value
                        .get("children")
                        .unwrap_or(&zero)
                        .as_array()
                        .unwrap_or(&empty);

                    if !contents.is_empty() {
                        CollapsingHeader::new(value.get("name").unwrap().as_str().unwrap())
                            .default_open(false)
                            .show(ui, |ui| {
                                for v in contents {
                                    self.add_dir(ui, v);
                                }
                            });
                    }
                }
                Some("file") => {
                    let name = value.get("name").unwrap().as_str().unwrap();
                    let link = ui.link(name);
                    if link.clicked() {
                        let path = value.get("path").unwrap().as_str().unwrap();
                        self.status = if let Ok(()) = self.load(path) {
                            format!("[OK] {} loaded!", name)
                        } else {
                            format!("[ERROR] Can't load {}!", name)
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        self.add_dir(ui, &self.json.to_owned());
    }
}
