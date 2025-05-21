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
    use wasm_bindgen_futures::JsFuture;
    use web_sys::js_sys::Uint8Array;
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

        let window = web_sys::window().unwrap();
        let request = Request::new_with_str_and_init(&filename, &opts).unwrap();

        wasm_bindgen_futures::spawn_local(async move {
            let resp_value = JsFuture::from(window.fetch_with_request(&request))
                .await
                .expect("resp");
            let resp: Response = resp_value.dyn_into().expect("respons");
            let buffer = JsFuture::from(resp.array_buffer().unwrap())
                .await
                .expect("val");

            let vec = Uint8Array::new(&buffer).to_vec();
        });

        // wasm_bindgen_futures::future_to_promise(window.fetch_with_request(&request)).await;

        Vec::new()
    }
}
#[cfg(target_arch = "wasm32")]
pub use fetch::{files, open};
