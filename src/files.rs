use std::fs::File;

use eframe::egui::{CollapsingHeader, Ui};
use egui_ltreeview::{TreeView, TreeViewBuilder};
use serde_json::Value;

pub struct Files;

impl Files {
    // pub fn new() -> Self {
    //     let file = File::open("/Users/fazibear/dev/vicious/c64Music.json")
    //         .expect("file should open read only");
    //     let json: Value = serde_json::from_reader(file).expect("file should be proper JSON");
    //     //let json: Value = Value::from(1);
    //     Self { json }
    // }

    pub fn add_dir(ui: &mut Ui, value: &Value) {
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
                    } else {
                        //println!("{:?}", value);
                    }
                }
                Some("file") => {
                    let link = ui.link(value.get("name").unwrap().as_str().unwrap());
                    if link.clicked() {}
                }
                x => {
                    println!("{:?}", x);
                }
            }
        } else {
            println!("no type");
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        let json = self.json.clone();
        self.add_dir(ui, &json);
    }
}
