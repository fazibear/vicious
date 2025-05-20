use std::fs::File;

use serde_json::Value;

pub fn load() -> Value {
    let json_path = concat!(env!("CARGO_MANIFEST_DIR"), "/C64Music.json");
    let file = File::open(json_path).expect("file should open");
    let json: Value = serde_json::from_reader(file).expect("file should be proper JSON");
    json
}
