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
        Triangle {
            vertices: [
                Vertex {
                    position: [0.0, 0.5, 0.0],
                    color: [1.0, 0.0, 0.0],
                },
                Vertex {
                    position: [-0.5, -0.5, 0.0],
                    color: [0.0, 1.0, 0.0],
                },
                Vertex {
                    position: [0.5, -0.5, 0.0],
                    color: [0.0, 0.0, 1.0],
                },
            ],
            transform: Transform::new(),
        }
    }
}

impl Renderable for Triangle {
    fn update(&mut self, delta: f32) {
        self.transform.rotate_degrees(0.0, 0.0, 60.0 * delta);
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
