static CURRENT_DIR: &str = env!("CARGO_MANIFEST_DIR");

#[cfg(not(target_arch = "wasm32"))]
mod fs {
    use serde_json::Value;
    use std::fs::File;

    pub fn files() -> Value {
        let json_path = format!("{}/C64Music.json", super::CURRENT_DIR);
        let file = File::open(json_path).expect("file should open");
        let json: Value = serde_json::from_reader(file).expect("file should be proper JSON");
        json
    }

    pub fn open(filename: &str) -> Vec<u8> {
        let full = format!("{}/{}", super::CURRENT_DIR, filename);
        let path = std::path::Path::new(&full);
        let data = std::fs::read(path).unwrap();
        data
    }
}
#[cfg(not(target_arch = "wasm32"))]
pub use fs::{files, open};

#[cfg(target_arch = "wasm32")]
mod fetch {
    use serde_json::Value;
    use web_sys::wasm_bindgen::JsCast;
    use web_sys::{console, Request, RequestInit, RequestMode, Response};

    pub fn files() -> Value {
        let file = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/C64Music.json"));
        let json: Value = serde_json::from_str(file).expect("file should be proper JSON");
        json
    }
    pub fn open(filename: &str) -> Vec<u8> {
        let opts = RequestInit::new();
        opts.set_method("GET");
        opts.set_mode(RequestMode::Cors);

        let request = Request::new_with_str_and_init(&filename, &opts).unwrap();
        let window = web_sys::window().unwrap();
        //let resp_value = window.fetch_with_request(&request);
        //let resp: Response = resp_value.dyn_into().unwrap();
        console::log_1(&"hello".into());
        //let bin: Vec<u8> = resp.text().unwrap().array_buffer();

        Vec::new()
    }
}
#[cfg(target_arch = "wasm32")]
pub use fetch::{files, open};
