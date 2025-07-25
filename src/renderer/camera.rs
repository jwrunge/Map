//! Camera and projection system
//!
//! Handles view and projection matrices for 3D rendering

use glam::{Mat4, Vec3};

/// Camera with projection and view matrix management
pub struct Camera {
    pub view_matrix: Mat4,
    pub projection_matrix: Mat4,
    pub aspect_ratio: f32,
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
}

impl Camera {
    pub fn new(aspect_ratio: f32) -> Self {
        let mut camera = Self {
            view_matrix: Mat4::IDENTITY,
            projection_matrix: Mat4::IDENTITY,
            aspect_ratio,
            position: Vec3::new(0.0, 0.0, 1.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
        };

        camera.update_matrices();
        camera
    }

    /// Create an orthographic camera suitable for 2D rendering
    pub fn orthographic_2d(aspect_ratio: f32) -> Self {
        let mut camera = Self::new(aspect_ratio);
        camera.position = Vec3::new(0.0, 0.0, 1.0);
        camera.target = Vec3::ZERO;
        camera.update_matrices();
        camera
    }

    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
        self.update_matrices();
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
        self.update_matrices();
    }

    pub fn look_at(&mut self, target: Vec3) {
        self.target = target;
        self.update_matrices();
    }

    pub fn get_view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix * self.view_matrix
    }

    fn update_matrices(&mut self) {
        // For 2D triangle rendering, use orthographic projection
        self.projection_matrix = Mat4::orthographic_rh(
            -self.aspect_ratio,
            self.aspect_ratio, // left, right
            -1.0,
            1.0, // bottom, top
            -1.0,
            1.0, // near, far
        );

        self.view_matrix = Mat4::look_at_rh(self.position, self.target, self.up);
    }
}
