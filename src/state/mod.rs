use web_time::Instant;
use winit::window::Window;

use crate::renderer::{Renderer, camera::ProjectionMode};
use crate::renderable::Renderable;
use crate::scene::Scene;

pub struct State {
    pub window: std::sync::Arc<Window>,
    pub renderer: Renderer,
    pub scene: Scene,
    pub projection_mode: ProjectionMode,
    last_frame_time: Instant,
}

impl State {
    pub async fn new(window: std::sync::Arc<Window>) -> Result<State, Box<dyn std::error::Error>> {
        let renderer = Renderer::new(window.clone()).await?;
        let mut scene = Scene::new();

        // Create three triangles using Scene primitive functions - spread across depth
        let id1 = scene.create_triangle_at(0.3, glam::Vec3::new(-2.0, 1.0, -4.0));
        log::info!("Created triangle1 at (-2.0, 1.0, -4.0) with ID {}", id1);

        let id2 = scene.create_triangle_at(0.3, glam::Vec3::new(2.0, 1.0, -4.0));
        log::info!("Created triangle2 at (2.0, 1.0, -4.0) with ID {}", id2);

        let id3 = scene.create_triangle_at(0.3, glam::Vec3::new(0.0, 1.0, -6.0));
        log::info!("Created triangle3 at (0.0, 1.0, -6.0) with ID {}", id3);

        // Create two quads using Scene primitive functions - lower and spread out
        let id4 = scene.create_quad_at(0.3, glam::Vec3::new(-3.0, -1.0, -3.0));
        log::info!("Created quad1 at (-3.0, -1.0, -3.0) with ID {}", id4);

        let id5 = scene.create_quad_at(0.3, glam::Vec3::new(3.0, -1.0, -5.0));
        log::info!("Created quad2 at (3.0, -1.0, -5.0) with ID {}", id5);

        // Create a cube using Scene primitive functions - center but back
        let id6 = scene.create_cube_at(0.3, glam::Vec3::new(0.0, 0.0, -8.0));
        log::info!("Created cube1 at (0.0, 0.0, -8.0) with ID {}", id6);

        // Create new primitive types to showcase the expanded library - well spaced in 3D
        
        // Circle - positioned to the left and slightly forward
        let mut circle = crate::renderable::Circle::new(0.4, 12);
        circle.transform_set_position(glam::Vec3::new(-4.0, 0.5, -1.0));
        let id7 = scene.add_circle(circle);
        log::info!("Created circle at (-4.0, 0.5, -1.0) with ID {}", id7);

        // Cylinder - positioned to the right and mid-depth
        let mut cylinder = crate::renderable::Cylinder::new(0.2, 0.8, 12);
        cylinder.transform_set_position(glam::Vec3::new(4.0, 0.0, -4.5));
        let id8 = scene.add_cylinder(cylinder);
        log::info!("Created cylinder at (4.0, 0.0, -4.5) with ID {}", id8);

        // Cone - positioned behind and slightly left
        let mut cone = crate::renderable::Cone::new(0.3, 0.6, 10);
        cone.transform_set_position(glam::Vec3::new(-1.5, -0.5, -7.0));
        let id9 = scene.add_cone(cone);
        log::info!("Created cone at (-1.5, -0.5, -7.0) with ID {}", id9);

        // Sphere - positioned above and forward from center
        let mut sphere = crate::renderable::Sphere::new(0.25, 20, 20);
        sphere.transform_set_position(glam::Vec3::new(1.0, 2.0, -2.5));
        let id10 = scene.add_sphere(sphere);
        log::info!("Created sphere at (1.0, 2.0, -2.5) with ID {}", id10);        log::info!(
            "Total objects in scene: {} triangles, {} quads, {} cubes, {} circles, {} cylinders, {} cones, {} spheres",
            scene.triangle_count(),
            scene.quad_count(),
            scene.cube_count(),
            scene.circle_count(),
            scene.cylinder_count(),
            scene.cone_count(),
            scene.sphere_count()
        );

        Ok(Self {
            window,
            renderer,
            scene,
            projection_mode: ProjectionMode::Perspective,
            last_frame_time: Instant::now(),
        })
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
            .camera_mut()
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
        let (triangles, quads, cubes, circles, cylinders, cones, spheres) = self.scene.get_all_renderables();

        // Log the object counts only at debug level to reduce console spam
        log::debug!(
            "Rendering scene: {} triangles, {} quads, {} cubes, {} circles, {} cylinders, {} cones, {} spheres",
            triangles.len(),
            quads.len(),
            cubes.len(),
            circles.len(),
            cylinders.len(),
            cones.len(),
            spheres.len()
        );

        // Render all object types in a single unified pass!
        // All primitives are now supported by the renderer
        self.renderer
            .render_mixed_objects(&triangles, &quads, &cubes, &circles, &cylinders, &cones, &spheres)?;

        Ok(())
    }
}
