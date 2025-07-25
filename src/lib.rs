mod app;
mod renderable;
mod renderer;
mod scene;
mod state;

use winit::event_loop::EventLoop;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
            console_error_panic_hook::set_once();
        } else {
            env_logger::init();
        }
    }

    log::info!("Starting application...");

    let event_loop = EventLoop::new().expect("Failed to create event loop");

    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let app = app::App::default();
            use winit::platform::web::EventLoopExtWebSys;
            log::info!("Starting WASM event loop...");
            event_loop.spawn_app(app);
        } else {
            let mut app = app::App::default();
            if let Err(e) = event_loop.run_app(&mut app) {
                log::error!("Event loop error: {:?}", e);
            }
        }
    }
}

pub use app::App;
pub use state::State;
