//! Main application logic with improved architecture
//!
//! This module demonstrates the new structure with separated concerns

use crate::renderable::Triangle;
use crate::renderer::Renderer;
use crate::scene::Scene;
use std::time::Instant;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

/// Main application with scene-based architecture
pub struct App {
    pub renderer: Option<Renderer>,
    pub scene: Scene,
    pub window: Option<std::sync::Arc<Window>>,
    pub last_frame_time: Instant,
    pub renderer_ready: bool,

    #[cfg(target_arch = "wasm32")]
    pub renderer_holder: Option<Rc<RefCell<Option<Renderer>>>>,
}

impl Default for App {
    fn default() -> Self {
        let mut scene = Scene::new();

        // Add a triangle to the scene
        let triangle = Triangle::new();
        scene.add_triangle(triangle);

        Self {
            renderer: None,
            scene,
            window: None,
            last_frame_time: Instant::now(),
            renderer_ready: false,

            #[cfg(target_arch = "wasm32")]
            renderer_holder: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            #[allow(unused_mut)]
            let mut window_attributes =
                Window::default_attributes().with_title("wgpu Graphics Engine");

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
                    log::info!("Creating renderer for native...");
                    let renderer = pollster::block_on(Renderer::new(window));
                    log::info!("Native renderer created successfully");
                    self.renderer = Some(renderer);
                } else {
                    log::info!("WASM window created, renderer will be initialized on demand");
                }
            }
        }

        // For WASM, initialize renderer asynchronously
        #[cfg(target_arch = "wasm32")]
        {
            if self.renderer.is_none() && self.window.is_some() && !self.renderer_ready {
                log::info!("Starting WASM renderer initialization...");
                let window = self.window.clone().unwrap();
                let renderer_holder = Rc::new(RefCell::new(None));
                self.renderer_holder = Some(renderer_holder.clone());

                wasm_bindgen_futures::spawn_local(async move {
                    log::info!("Creating WASM renderer...");
                    let renderer = Renderer::new(window).await;
                    log::info!("WASM renderer created successfully!");
                    *renderer_holder.borrow_mut() = Some(renderer);
                });

                self.renderer_ready = true;
                log::info!("WASM renderer initialization started");
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                ..
            } => event_loop.exit(),

            WindowEvent::Resized(physical_size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(physical_size);
                }
            }

            WindowEvent::RedrawRequested => {
                // For WASM, check if async renderer is ready
                #[cfg(target_arch = "wasm32")]
                {
                    let should_clear_holder =
                        if self.renderer.is_none() && self.renderer_holder.is_some() {
                            let holder = self.renderer_holder.as_ref().unwrap();
                            if let Ok(mut borrowed) = holder.try_borrow_mut() {
                                if let Some(mut renderer) = borrowed.take() {
                                    log::info!("WASM renderer successfully received!");

                                    if let Some(window) = &self.window {
                                        let size = window.inner_size();
                                        renderer.resize(size);
                                        log::info!("WASM renderer resized to {:?}", size);
                                    }

                                    self.renderer = Some(renderer);
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
                        self.renderer_holder = None;
                        if let Some(window) = &self.window {
                            window.request_redraw();
                        }
                    }
                }

                // Calculate delta time
                let current_time = Instant::now();
                let delta_time = (current_time - self.last_frame_time).as_secs_f32();
                self.last_frame_time = current_time;

                if let Some(renderer) = &mut self.renderer {
                    // Update scene
                    self.scene.update(delta_time);

                    // Render scene
                    match self
                        .scene
                        .render_triangles(|triangle| renderer.render(triangle))
                    {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            renderer.resize(renderer.gpu.size)
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
