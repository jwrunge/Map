//! Renderable objects and graphics primitives
//!
//! This module provides the core traits and implementations for objects
//! that can be rendered to the screen.

mod transforms;
pub mod vertex;

pub use transforms::Transform;
pub use vertex::{Vertex, VertexProvider};

/// Trait for objects that can be rendered and updated
pub trait Renderable {
    fn update(&mut self, delta: f32);
    fn get_transform(&self) -> &Transform;
}

pub struct Triangle {
    vertices: [Vertex; 3],
    pub transform: Transform,
}

impl Triangle {
    pub fn new() -> Self {
        Self::with_scale(1.0)
    }

    pub fn with_scale(scale: f32) -> Self {
        let height = scale * 3.0_f32.sqrt() / 2.0;
        let top_y = (2.0 / 3.0) * height;
        let bottom_y = -(1.0 / 3.0) * height;

        Triangle {
            vertices: [
                // Top vertex (red)
                Vertex {
                    position: [0.0, top_y, 0.0],
                    color: [1.0, 0.0, 0.0],
                },
                // Bottom-left vertex (green)
                Vertex {
                    position: [-scale / 2.0, bottom_y, 0.0],
                    color: [0.0, 1.0, 0.0],
                },
                // Bottom-right vertex (blue)
                Vertex {
                    position: [scale / 2.0, bottom_y, 0.0],
                    color: [0.0, 0.0, 1.0],
                },
            ],
            transform: Transform::new(),
        }
    }
}

impl Renderable for Triangle {
    fn update(&mut self, delta: f32) {
        // Re-enable rotation
        self.transform.rotate_degrees(0.0, 0.0, 15.0 * delta);
    }

    fn get_transform(&self) -> &Transform {
        &self.transform
    }
}

impl VertexProvider for Triangle {
    fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }
}
