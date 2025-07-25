use web_time::Instant;
use winit::window::Window;

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

        // Create three triangles using Scene primitive functions
        let id1 = scene.create_triangle_at(0.3, glam::Vec3::new(-0.5, 0.5, -2.0));
        log::info!("Created triangle1 at (-0.5, 0.5, -2.0) with ID {}", id1);

        let id2 = scene.create_triangle_at(0.3, glam::Vec3::new(0.5, 0.5, 0.0));
        log::info!("Created triangle2 at (0.5, 0.5, 0.0) with ID {}", id2);

        let id3 = scene.create_triangle_at(0.3, glam::Vec3::new(0.0, 0.5, 2.0));
        log::info!("Created triangle3 at (0.0, 0.5, 2.0) with ID {}", id3);

        // Create two quads using Scene primitive functions
        let id4 = scene.create_quad_at(0.3, glam::Vec3::new(-1.0, -0.5, 0.0));
        log::info!("Created quad1 at (-1.0, -0.5, 0.0) with ID {}", id4);

        let id5 = scene.create_quad_at(0.3, glam::Vec3::new(1.0, -0.5, 0.0));
        log::info!("Created quad2 at (1.0, -0.5, 0.0) with ID {}", id5);

        // Create a cube using Scene primitive functions
        let id6 = scene.create_cube_at(0.3, glam::Vec3::new(0.0, -0.5, -1.0));
        log::info!("Created cube1 at (0.0, -0.5, -1.0) with ID {}", id6);

        log::info!(
            "Total objects in scene: {} triangles, {} quads, {} cubes",
            scene.triangle_count(),
            scene.quad_count(),
            scene.cube_count()
        );

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
        // Get all objects from the scene
        let (triangles, quads, cubes) = self.scene.get_all_renderables();

        // Log the object counts to show the architecture is working
        log::info!(
            "Rendering scene: {} triangles, {} quads, {} cubes",
            triangles.len(),
            quads.len(),
            cubes.len()
        );

        // Render all object types in a single unified pass!
        self.renderer
            .render_mixed_objects(&triangles, &quads, &cubes)?;

        Ok(())
    }
}
