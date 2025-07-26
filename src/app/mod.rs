use crate::state::State;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

pub struct App {
    pub state: Option<State>,
    pub window: Option<std::sync::Arc<Window>>,
    #[cfg(target_arch = "wasm32")]
    pub state_ready: bool,
    #[cfg(target_arch = "wasm32")]
    pub state_holder: Option<Rc<RefCell<Option<State>>>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            state: None,
            window: None,
            #[cfg(target_arch = "wasm32")]
            state_ready: false,
            #[cfg(target_arch = "wasm32")]
            state_holder: None,
        }
    }
}
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            #[allow(unused_mut)]
            let mut window_attributes = Window::default_attributes().with_title("wgpu Triangle");

            #[cfg(target_arch = "wasm32")]
            {
                use winit::platform::web::WindowAttributesExtWebSys;
                if let Some(canvas) = web_sys::window()
                    .and_then(|win| win.document())
                    .and_then(|doc| doc.get_element_by_id("wasm-canvas"))
                    .and_then(|element| element.dyn_into::<web_sys::HtmlCanvasElement>().ok())
                {
                    window_attributes = window_attributes.with_canvas(Some(canvas));
                }
            }

            let window = std::sync::Arc::new(event_loop.create_window(window_attributes).unwrap());
            self.window = Some(window.clone());

            cfg_if::cfg_if! {
                if #[cfg(not(target_arch = "wasm32"))] {
                    log::info!("Creating wgpu state for native...");
                    match pollster::block_on(State::new(window)) {
                        Ok(state) => {
                            log::info!("Native state created successfully");
                            self.state = Some(state);
                        }
                        Err(e) => {
                            log::error!("Failed to create native state: {}", e);
                            // For now, continue without state - could add more sophisticated error handling
                        }
                    }
                } else {
                    log::info!("WASM window created, GPU state will be initialized on demand");
                }
            }
        }

        // For WASM, properly initialize state with channel communication
        #[cfg(target_arch = "wasm32")]
        {
            if self.state.is_none() && self.window.is_some() && !self.state_ready {
                log::info!("Starting WASM GPU state initialization...");
                let window = self.window.clone().unwrap();
                let state_holder = Rc::new(RefCell::new(None));
                self.state_holder = Some(state_holder.clone());

                wasm_bindgen_futures::spawn_local(async move {
                    log::info!("Creating WASM GPU state...");
                    match State::new(window).await {
                        Ok(state) => {
                            log::info!("WASM GPU state created successfully!");
                            *state_holder.borrow_mut() = Some(state);
                        }
                        Err(e) => {
                            log::error!("Failed to create WASM GPU state: {}", e);
                        }
                    }
                });

                self.state_ready = true;
                log::info!("WASM state initialization started");
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == winit::event::ElementState::Pressed {
                    match event.logical_key {
                        winit::keyboard::Key::Named(winit::keyboard::NamedKey::Space) => {
                            if let Some(ref mut state) = self.state {
                                state.toggle_projection();
                            }
                        }
                        winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape) => {
                            event_loop.exit();
                        }
                        _ => {}
                    }
                }
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(physical_size) => {
                if let Some(state) = &mut self.state {
                    state.resize(physical_size);
                }
            }
            WindowEvent::RedrawRequested => {
                // For WASM, check if async state is ready
                #[cfg(target_arch = "wasm32")]
                {
                    let should_clear_holder = if self.state.is_none() && self.state_holder.is_some()
                    {
                        let holder = self.state_holder.as_ref().unwrap();
                        if let Ok(mut borrowed) = holder.try_borrow_mut() {
                            if let Some(mut state) = borrowed.take() {
                                log::info!("WASM GPU state successfully received!");
                                // Ensure the state is properly sized and configured for the current window
                                if let Some(window) = &self.window {
                                    let size = window.inner_size();
                                    state.resize(size);
                                    log::info!("WASM state resized to {:?}", size);

                                    // Force a surface reconfiguration for WASM
                                    #[cfg(target_arch = "wasm32")]
                                    {
                                        state.renderer.gpu.surface.configure(&state.renderer.gpu.device, &state.renderer.gpu.config);
                                        log::info!("WASM surface reconfigured");
                                    }
                                }
                                self.state = Some(state);
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    if should_clear_holder {
                        self.state_holder = None;
                        // Request an immediate redraw when state becomes available
                        if let Some(window) = &self.window {
                            window.request_redraw();
                        }
                    }
                }

                if let Some(state) = &mut self.state {
                    state.update();
                    match state.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            state.resize(state.renderer.gpu.size)
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            log::error!("OutOfMemory");
                            event_loop.exit();
                        }
                        Err(wgpu::SurfaceError::Timeout) => {
                            log::warn!("Surface timeout")
                        }
                        Err(wgpu::SurfaceError::Other) => {
                            log::error!("Other surface error");
                        }
                    }
                } else {
                    // For WASM, occasionally log status while waiting
                    #[cfg(target_arch = "wasm32")]
                    {
                        static mut FRAME_COUNT: u32 = 0;
                        unsafe {
                            FRAME_COUNT += 1;
                            if FRAME_COUNT % 120 == 1 {
                                // Log every ~2 seconds at 60fps
                                if self.state_holder.is_some() {
                                    log::info!(
                                        "WASM: Still waiting for GPU state initialization..."
                                    );
                                } else if !self.state_ready {
                                    log::info!("WASM: Waiting for window setup...");
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
