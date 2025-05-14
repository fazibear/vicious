mod app;
mod memory;
mod player;
mod sound;

use app::App;
use eframe::egui::ViewportBuilder;
use eframe::NativeOptions;
use memory::PlayerMemory;
use player::Player;
use sound::Sound;

fn main() -> eframe::Result {
    pretty_env_logger::init();
    let options = NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([640.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Ok(Box::<App>::default())),
    )
}
