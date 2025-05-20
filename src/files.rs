use anyhow::Result;
use serde_json::Value;
use std::fs::File;

pub fn files() -> Value {
    let json_path = concat!(env!("CARGO_MANIFEST_DIR"), "/C64Music.json");
    let file = File::open(json_path).expect("file should open");
    let json: Value = serde_json::from_reader(file).expect("file should be proper JSON");
    json
}

pub fn open(filename: &str) -> Result<Vec<u8>> {
    let path = std::path::Path::new(filename);
    let data = std::fs::read(path)?;
    Ok(data)
}
