mod app;
mod files;
mod output;
mod sid_player;

use app::App;
use eframe::egui::ViewportBuilder;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    use std::time::{Duration, Instant};

    pretty_env_logger::init();
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([640.0, 480.0]),
        ..Default::default()
    };

    let app = Box::<App>::default();
    let sid_player_thread = app.sid_player.clone();
    let mut last_step = Instant::now();

    std::thread::spawn(move || loop {
        if last_step.elapsed() < Duration::from_millis(20) {
            continue;
        }

        last_step = Instant::now();

        sid_player_thread.lock().step();
    });

    eframe::run_native("Vicious", options, Box::new(|_cc| Ok(app)))
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|_cc| Ok(Box::<App>::default())),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}
