use web_time::Instant;
use winit::window::Window;

use crate::renderable::{Renderable, Triangle};
use crate::renderer::Renderer;
use crate::scene::Scene;

#[derive(Copy, Clone)]
pub enum ProjectionMode {
    Orthographic,
    Perspective,
}

pub struct State {
    pub window: std::sync::Arc<Window>,
    pub renderer: Renderer,
    pub scene: Scene,
    pub projection_mode: ProjectionMode,
    last_frame_time: Instant,
}

impl State {
    pub async fn new(window: std::sync::Arc<Window>) -> State {
        let renderer = Renderer::new(window.clone()).await;
        let mut scene = Scene::new();

        // Create some triangles to demonstrate the scene system with Z depth testing
        let mut triangle1 = Triangle::with_scale(0.3);
        triangle1.transform_translate(-0.5, 0.0, -5.0); // Far back
        let id1 = scene.add_triangle(triangle1);
        log::info!("Added triangle1 at (-0.5, 0, -5.0) with ID {}", id1);

        let mut triangle2 = Triangle::with_scale(0.3);
        triangle2.transform_translate(0.5, 0.0, 0.0); // Center depth
        triangle2.transform_rotate_radians(0.0, 0.0, std::f32::consts::PI / 6.0); // 30 degrees
        let id2 = scene.add_triangle(triangle2);
        log::info!("Added triangle2 at (0.5, 0, 0.0) with ID {}", id2);

        let mut triangle3 = Triangle::with_scale(0.3);
        triangle3.transform_translate(0.0, 0.4, 8.0); // Far forward
        triangle3.transform_rotate_radians(0.0, 0.0, std::f32::consts::PI / 3.0); // 60 degrees
        let id3 = scene.add_triangle(triangle3);
        log::info!("Added triangle3 at (0, 0.4, 8.0) with ID {}", id3);

        log::info!("Total triangles in scene: {}", scene.triangle_count());

        Self {
            window,
            renderer,
            scene,
            projection_mode: ProjectionMode::Perspective,
            last_frame_time: Instant::now(),
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.renderer.resize(new_size);
    }

    pub fn toggle_projection(&mut self) {
        self.projection_mode = match self.projection_mode {
            ProjectionMode::Orthographic => ProjectionMode::Perspective,
            ProjectionMode::Perspective => ProjectionMode::Orthographic,
        };

        self.renderer
            .camera
            .set_projection_mode(self.projection_mode);
    }

    pub fn update(&mut self) {
        // Calculate real delta time
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        // Cap delta time to avoid huge jumps (e.g., when debugging or app is backgrounded)
        let delta_time = delta_time.min(0.1); // Max 100ms per frame

        // Update the scene with real delta time
        self.scene.update(delta_time);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // Render all triangles in the scene using dynamic batch rendering
        self.scene
            .render_triangles_batch(|triangles| self.renderer.render_batch_dynamic(triangles))
    }
}
