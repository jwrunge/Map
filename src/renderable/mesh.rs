//! 3D mesh definitions and primitive geometry
//!
//! This module defines the Mesh trait for 3D geometry and provides
//! factory functions for creating common primitive shapes.

use super::{Vertex, VertexProvider};

/// Trait for 3D mesh geometry
///
/// Separates geometry definition from rendering behavior.
/// Meshes define the shape and vertex data, while Renderable handles transforms and animations.
pub trait Mesh: VertexProvider {
    /// Get the name/type of this mesh for debugging
    fn mesh_type(&self) -> &'static str;

    /// Get the approximate bounds of this mesh (for culling, etc.)
    /// Returns (min, max) corners of axis-aligned bounding box
    fn bounds(&self) -> (glam::Vec3, glam::Vec3);

    /// Check if this mesh uses indexed rendering (for future optimization)
    fn is_indexed(&self) -> bool {
        false
    }

    /// Get index data if this mesh uses indexed rendering
    fn index_data(&self) -> Option<&[u16]> {
        None
    }
}

/// Triangle mesh (our existing implementation)
#[derive(Debug, Clone)]
pub struct TriangleMesh {
    vertices: [Vertex; 3],
    scale: f32,
}

impl TriangleMesh {
    /// Create a new equilateral triangle mesh with the given scale
    pub fn new(scale: f32) -> Self {
        // Create equilateral triangle vertices (pointing up)
        let height = (3.0_f32.sqrt() / 2.0) * scale;
        let half_base = scale / 2.0;

        let vertices = [
            // Top vertex (red)
            Vertex {
                position: [0.0, height * (2.0 / 3.0), 0.0],
                color: [1.0, 0.0, 0.0],
            },
            // Bottom left vertex (green)
            Vertex {
                position: [-half_base, -height * (1.0 / 3.0), 0.0],
                color: [0.0, 1.0, 0.0],
            },
            // Bottom right vertex (blue)
            Vertex {
                position: [half_base, -height * (1.0 / 3.0), 0.0],
                color: [0.0, 0.0, 1.0],
            },
        ];

        Self { vertices, scale }
    }
}

impl VertexProvider for TriangleMesh {
    fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    fn vertex_count(&self) -> usize {
        3
    }

    fn buffer_contents(&self) -> &[u8] {
        bytemuck::cast_slice(&self.vertices)
    }
}

impl Mesh for TriangleMesh {
    fn mesh_type(&self) -> &'static str {
        "Triangle"
    }

    fn bounds(&self) -> (glam::Vec3, glam::Vec3) {
        let half_scale = self.scale / 2.0;
        let height = (3.0_f32.sqrt() / 2.0) * self.scale;
        (
            glam::Vec3::new(-half_scale, -height * (1.0 / 3.0), 0.0),
            glam::Vec3::new(half_scale, height * (2.0 / 3.0), 0.0),
        )
    }
}

/// Quad mesh (two triangles forming a rectangle)
#[derive(Debug, Clone)]
pub struct QuadMesh {
    vertices: [Vertex; 6], // Two triangles = 6 vertices
    width: f32,
    height: f32,
}

impl QuadMesh {
    /// Create a new quad mesh with the given dimensions
    pub fn new(width: f32, height: f32) -> Self {
        let half_width = width / 2.0;
        let half_height = height / 2.0;

        let vertices = [
            // First triangle (top-left, bottom-left, top-right)
            Vertex {
                position: [-half_width, half_height, 0.0],
                color: [1.0, 0.0, 0.0], // Red
            },
            Vertex {
                position: [-half_width, -half_height, 0.0],
                color: [0.0, 1.0, 0.0], // Green
            },
            Vertex {
                position: [half_width, half_height, 0.0],
                color: [0.0, 0.0, 1.0], // Blue
            },
            // Second triangle (top-right, bottom-left, bottom-right)
            Vertex {
                position: [half_width, half_height, 0.0],
                color: [0.0, 0.0, 1.0], // Blue
            },
            Vertex {
                position: [-half_width, -half_height, 0.0],
                color: [0.0, 1.0, 0.0], // Green
            },
            Vertex {
                position: [half_width, -half_height, 0.0],
                color: [1.0, 1.0, 0.0], // Yellow
            },
        ];

        Self {
            vertices,
            width,
            height,
        }
    }
}

impl VertexProvider for QuadMesh {
    fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    fn vertex_count(&self) -> usize {
        6
    }

    fn buffer_contents(&self) -> &[u8] {
        bytemuck::cast_slice(&self.vertices)
    }
}

impl Mesh for QuadMesh {
    fn mesh_type(&self) -> &'static str {
        "Quad"
    }

    fn bounds(&self) -> (glam::Vec3, glam::Vec3) {
        let half_width = self.width / 2.0;
        let half_height = self.height / 2.0;
        (
            glam::Vec3::new(-half_width, -half_height, 0.0),
            glam::Vec3::new(half_width, half_height, 0.0),
        )
    }
}

/// Cube mesh (12 triangles forming a cube)
#[derive(Debug, Clone)]
pub struct CubeMesh {
    vertices: [Vertex; 36], // 6 faces * 2 triangles * 3 vertices = 36
    size: f32,
}

impl CubeMesh {
    /// Create a new cube mesh with the given size
    pub fn new(size: f32) -> Self {
        let half_size = size / 2.0;

        // Define colors for each face
        let colors = [
            [1.0, 0.0, 0.0], // Front face - Red
            [0.0, 1.0, 0.0], // Back face - Green
            [0.0, 0.0, 1.0], // Top face - Blue
            [1.0, 1.0, 0.0], // Bottom face - Yellow
            [1.0, 0.0, 1.0], // Right face - Magenta
            [0.0, 1.0, 1.0], // Left face - Cyan
        ];

        let mut vertices = [Vertex {
            position: [0.0, 0.0, 0.0],
            color: [0.0, 0.0, 0.0],
        }; 36];

        let mut vertex_index = 0;

        // Define cube vertices systematically
        // Using standard cube vertex positions
        let vertices_pos = [
            // Front face vertices (z = +half_size)
            [-half_size, -half_size, half_size], // 0: front bottom left
            [half_size, -half_size, half_size],  // 1: front bottom right
            [half_size, half_size, half_size],   // 2: front top right
            [-half_size, half_size, half_size],  // 3: front top left
            // Back face vertices (z = -half_size)
            [-half_size, -half_size, -half_size], // 4: back bottom left
            [half_size, -half_size, -half_size],  // 5: back bottom right
            [half_size, half_size, -half_size],   // 6: back top right
            [-half_size, half_size, -half_size],  // 7: back top left
        ];

        // Define faces with proper counter-clockwise winding when viewed from outside
        let face_indices = [
            // Front face (0, 1, 2, 3) - looking at +Z
            [0, 1, 2, 0, 2, 3], // CCW: bottom-left, bottom-right, top-right, then bottom-left, top-right, top-left
            // Back face (5, 4, 7, 6) - looking at -Z (reversed order for CCW)
            [5, 4, 7, 5, 7, 6], // CCW when viewed from outside
            // Top face (3, 2, 6, 7) - looking down at +Y
            [3, 2, 6, 3, 6, 7], // CCW when viewed from above
            // Bottom face (4, 5, 1, 0) - looking up at -Y (reversed order for CCW)
            [4, 5, 1, 4, 1, 0], // CCW when viewed from below
            // Right face (1, 5, 6, 2) - looking at +X
            [1, 5, 6, 1, 6, 2], // CCW when viewed from outside
            // Left face (4, 0, 3, 7) - looking at -X (reversed order for CCW)
            [4, 0, 3, 4, 3, 7], // CCW when viewed from outside
        ];

        let mut vertex_index = 0;

        // Generate vertices for each face
        for (face_idx, indices) in face_indices.iter().enumerate() {
            let color = colors[face_idx];

            // Each face has 6 vertices (2 triangles)
            for &vertex_idx in indices.iter() {
                vertices[vertex_index] = Vertex {
                    position: vertices_pos[vertex_idx],
                    color,
                };
                vertex_index += 1;
            }
        }

        Self { vertices, size }
    }
}

impl VertexProvider for CubeMesh {
    fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    fn vertex_count(&self) -> usize {
        36
    }

    fn buffer_contents(&self) -> &[u8] {
        bytemuck::cast_slice(&self.vertices)
    }
}

impl Mesh for CubeMesh {
    fn mesh_type(&self) -> &'static str {
        "Cube"
    }

    fn bounds(&self) -> (glam::Vec3, glam::Vec3) {
        let half_size = self.size / 2.0;
        (
            glam::Vec3::new(-half_size, -half_size, -half_size),
            glam::Vec3::new(half_size, half_size, half_size),
        )
    }
}
