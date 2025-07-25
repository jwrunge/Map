use winit::window::Window;

use crate::renderable::Triangle;
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
}

impl State {
    pub async fn new(window: std::sync::Arc<Window>) -> State {
        let renderer = Renderer::new(window.clone()).await;
        let mut scene = Scene::new();

        // Log camera info for debugging
        let size = window.inner_size();
        let aspect_ratio = size.width as f32 / size.height as f32;
        log::info!(
            "Window size: {}x{}, aspect ratio: {}",
            size.width,
            size.height,
            aspect_ratio
        );
        log::info!(
            "Camera bounds: X=[-{}, {}], Y=[-1, 1]",
            aspect_ratio,
            aspect_ratio
        );

        // Create some triangles to demonstrate the scene system with Z depth testing
        let mut triangle1 = Triangle::with_scale(0.3);
        triangle1.transform.translate_xyz(-0.5, 0.0, -5.0); // Far back
        let id1 = scene.add_triangle(triangle1);
        log::info!("Added triangle1 at (-0.5, 0, -5.0) with ID {}", id1);

        let mut triangle2 = Triangle::with_scale(0.3);
        triangle2.transform.translate_xyz(0.5, 0.0, 0.0); // Center depth
        triangle2
            .transform
            .rotate_radians(0.0, 0.0, std::f32::consts::PI / 6.0); // 30 degrees
        let id2 = scene.add_triangle(triangle2);
        log::info!("Added triangle2 at (0.5, 0, 0.0) with ID {}", id2);

        let mut triangle3 = Triangle::with_scale(0.3);
        triangle3.transform.translate_xyz(0.0, 0.4, 8.0); // Far forward
        triangle3
            .transform
            .rotate_radians(0.0, 0.0, std::f32::consts::PI / 3.0); // 60 degrees
        let id3 = scene.add_triangle(triangle3);
        log::info!("Added triangle3 at (0, 0.4, 8.0) with ID {}", id3);

        log::info!("Total triangles in scene: {}", scene.triangle_count());

        Self {
            window,
            renderer,
            scene,
            projection_mode: ProjectionMode::Perspective,
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

        match self.projection_mode {
            ProjectionMode::Orthographic => {
                log::info!("Switched to Orthographic projection - Z translation won't affect size");
            }
            ProjectionMode::Perspective => {
                log::info!("Switched to Perspective projection - Z translation affects size");
            }
        }
    }

    pub fn update(&mut self) {
        // Update the scene (animate triangles)
        self.scene.update(0.016); // ~60 FPS
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // Render all triangles in the scene using batch rendering
        self.scene
            .render_triangles_batch(|triangles| self.renderer.render_batch(triangles))
    }
}
