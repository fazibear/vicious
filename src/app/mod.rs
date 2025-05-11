use eframe::{
    egui,
    egui::{CentralPanel, Context, Slider},
    Frame,
};

pub struct App {
    name: String,
    age: u32,
}

impl Default for App {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            age: 42,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
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
