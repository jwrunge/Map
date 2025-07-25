//! Renderable objects and graphics primitives
//!
//! This module provides the core traits and implementations for objects
//! that can be rendered to the screen.

use glam;

pub mod mesh;
mod transforms;
pub mod vertex;

pub use mesh::{CubeMesh, Mesh, QuadMesh, TriangleMesh};
pub use transforms::Transform;
pub use vertex::{Vertex, VertexProvider};

/// Quad object (rectangle made of two triangles)
#[derive(Debug, Clone)]
pub struct Quad {
    mesh: QuadMesh,
    transform: Transform,
    is_dirty: bool,
}

impl Quad {
    /// Create a new quad with the given dimensions
    pub fn with_size(width: f32, height: f32) -> Self {
        Self {
            mesh: QuadMesh::new(width, height),
            transform: Transform::new(),
            is_dirty: true,
        }
    }
}

/// Cube object (3D cube made of 12 triangles)
#[derive(Debug, Clone)]
pub struct Cube {
    mesh: CubeMesh,
    transform: Transform,
    is_dirty: bool,
}

impl Cube {
    /// Create a new cube with the given size
    pub fn with_size(size: f32) -> Self {
        Self {
            mesh: CubeMesh::new(size),
            transform: Transform::new(),
            is_dirty: true,
        }
    }
}

// Implement Renderable for Quad
impl Renderable for Quad {
    fn get_transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }

    fn get_transform(&self) -> &Transform {
        &self.transform
    }

    fn get_matrix(&self) -> glam::Mat4 {
        self.transform.to_matrix()
    }

    fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    fn set_dirty(&mut self, dirty: bool) {
        self.is_dirty = dirty;
    }

    fn update(&mut self, delta: f32) {
        // Quad rotates around Y axis for variety
        self.transform_rotate_degrees(0.0, 20.0 * delta, 0.0);
    }
}

// Implement VertexProvider for Quad
impl VertexProvider for Quad {
    fn vertices(&self) -> &[Vertex] {
        self.mesh.vertices()
    }
}

// Implement Renderable for Cube
impl Renderable for Cube {
    fn get_transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }

    fn get_transform(&self) -> &Transform {
        &self.transform
    }

    fn get_matrix(&self) -> glam::Mat4 {
        self.transform.to_matrix()
    }

    fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    fn set_dirty(&mut self, dirty: bool) {
        self.is_dirty = dirty;
    }

    fn update(&mut self, delta: f32) {
        // Cube rotates around X and Y axes for visual appeal
        self.transform_rotate_degrees(30.0 * delta, 45.0 * delta, 0.0);
    }
}

// Implement VertexProvider for Cube
impl VertexProvider for Cube {
    fn vertices(&self) -> &[Vertex] {
        self.mesh.vertices()
    }
}

/// Trait for objects that can be rendered and updated
pub trait Renderable {
    fn is_dirty(&self) -> bool;
    fn set_dirty(&mut self, dirty: bool) -> ();
    fn update(&mut self, delta: f32);
    fn get_transform(&self) -> &Transform;
    fn get_transform_mut(&mut self) -> &mut Transform;
    fn get_matrix(&self) -> glam::Mat4;

    /// Mark object as clean after GPU update (called by renderer)
    fn mark_clean(&mut self) {
        self.set_dirty(false);
    }

    // Transform wrapper methods that automatically set dirty flag
    fn transform_translate(&mut self, x: f32, y: f32, z: f32) {
        self.get_transform_mut().translate_xyz(x, y, z);
        self.set_dirty(true);
    }

    fn transform_rotate_degrees(&mut self, x: f32, y: f32, z: f32) {
        self.get_transform_mut().rotate_degrees(x, y, z);
        self.set_dirty(true);
    }

    fn transform_rotate_radians(&mut self, x: f32, y: f32, z: f32) {
        self.get_transform_mut().rotate_radians(x, y, z);
        self.set_dirty(true);
    }

    fn transform_scale(&mut self, x: f32, y: f32, z: f32) {
        self.get_transform_mut().scale_xyz(x, y, z);
        self.set_dirty(true);
    }

    fn transform_set_position(&mut self, position: glam::Vec3) {
        self.get_transform_mut().set_position(position);
        self.set_dirty(true);
    }
}

pub struct Triangle {
    is_dirty: bool,
    vertices: [Vertex; 3],
    transform: Transform,
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
            is_dirty: true,
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
    fn get_transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }

    fn get_transform(&self) -> &Transform {
        &self.transform
    }

    fn get_matrix(&self) -> glam::Mat4 {
        self.transform.to_matrix()
    }

    fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    fn set_dirty(&mut self, dirty: bool) {
        self.is_dirty = dirty;
    }

    fn update(&mut self, delta: f32) {
        // Always animate (continuous rotation)
        self.transform_rotate_degrees(0.0, 0.0, 15.0 * delta);
        // Note: transform_rotate_degrees automatically sets dirty flag
        // The dirty flag will be cleared by the renderer after GPU update
    }
}

impl VertexProvider for Triangle {
    fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }
}

// Implement traits for references so they work with batch rendering
impl<T: Renderable> Renderable for &T {
    fn is_dirty(&self) -> bool {
        (**self).is_dirty()
    }

    fn set_dirty(&mut self, _dirty: bool) {
        // Cannot mutate through & reference - this is a limitation
        // For read-only batch rendering, this is acceptable
    }

    fn update(&mut self, _delta: f32) {
        // Cannot mutate through & reference
    }

    fn get_transform(&self) -> &Transform {
        (**self).get_transform()
    }

    fn get_transform_mut(&mut self) -> &mut Transform {
        // Cannot mutate through & reference - this won't work
        panic!("Cannot get mutable transform through & reference")
    }

    fn get_matrix(&self) -> glam::Mat4 {
        (**self).get_matrix()
    }
}

impl<T: VertexProvider> VertexProvider for &T {
    fn vertices(&self) -> &[Vertex] {
        (**self).vertices()
    }
}
