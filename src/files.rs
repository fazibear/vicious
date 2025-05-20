use anyhow::Result;
use serde_json::Value;
use std::fs::File;

static CURRENT_DIR: &str = env!("CARGO_MANIFEST_DIR");

pub fn files() -> Value {
    let json_path = format!("{}/{}", CURRENT_DIR, "/C64Music.json");
    let file = File::open(json_path).expect("file should open");
    let json: Value = serde_json::from_reader(file).expect("file should be proper JSON");
    json
}

pub fn open(filename: &str) -> Result<Vec<u8>> {
    let full = format!("{}/{}", CURRENT_DIR, filename);
    let path = std::path::Path::new(&full);
    let data = std::fs::read(path)?;
    Ok(data)
}
