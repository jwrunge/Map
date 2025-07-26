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

        let mut _vertex_index = 0;

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

/// Circle mesh (2D circle made of triangular segments)
#[derive(Debug, Clone)]
pub struct CircleMesh {
    vertices: Vec<Vertex>,
    radius: f32,
    segments: u32,
}

impl CircleMesh {
    /// Create a new circle mesh with the given radius and number of segments
    pub fn new(radius: f32, segments: u32) -> Self {
        let segments = segments.max(3); // Minimum 3 segments for a triangle
        let mut vertices = Vec::with_capacity((segments * 3) as usize);
        
        // Center vertex (white)
        let center = Vertex {
            position: [0.0, 0.0, 0.0],
            color: [1.0, 1.0, 1.0],
        };
        
        // Create triangular segments
        for i in 0..segments {
            let angle1 = (i as f32) * 2.0 * std::f32::consts::PI / (segments as f32);
            let angle2 = ((i + 1) as f32) * 2.0 * std::f32::consts::PI / (segments as f32);
            
            // Color varies around the circle (hue wheel effect)
            let hue1 = angle1 / (2.0 * std::f32::consts::PI);
            let hue2 = angle2 / (2.0 * std::f32::consts::PI);
            
            let color1 = hsv_to_rgb(hue1, 0.8, 1.0);
            let color2 = hsv_to_rgb(hue2, 0.8, 1.0);
            
            let vertex1 = Vertex {
                position: [radius * angle1.cos(), radius * angle1.sin(), 0.0],
                color: color1,
            };
            
            let vertex2 = Vertex {
                position: [radius * angle2.cos(), radius * angle2.sin(), 0.0],
                color: color2,
            };
            
            // Add triangle: center -> vertex1 -> vertex2
            vertices.push(center);
            vertices.push(vertex1);
            vertices.push(vertex2);
        }
        
        Self { vertices, radius, segments }
    }

    /// Get the radius of the circle
    pub fn radius(&self) -> f32 {
        self.radius
    }

    /// Get the number of segments in the circle
    pub fn segments(&self) -> u32 {
        self.segments
    }
}

impl VertexProvider for CircleMesh {
    fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }
    
    fn vertex_count(&self) -> usize {
        self.vertices.len()
    }
    
    fn buffer_contents(&self) -> &[u8] {
        bytemuck::cast_slice(&self.vertices)
    }
}

impl Mesh for CircleMesh {
    fn mesh_type(&self) -> &'static str {
        "Circle"
    }
    
    fn bounds(&self) -> (glam::Vec3, glam::Vec3) {
        (
            glam::Vec3::new(-self.radius, -self.radius, 0.0),
            glam::Vec3::new(self.radius, self.radius, 0.0),
        )
    }
}

/// Cylinder mesh (3D cylinder with circular cross-section)
#[derive(Debug, Clone)]
pub struct CylinderMesh {
    vertices: Vec<Vertex>,
    radius: f32,
    height: f32,
    segments: u32,
}

impl CylinderMesh {
    /// Create a new cylinder mesh with the given radius, height, and number of segments
    pub fn new(radius: f32, height: f32, segments: u32) -> Self {
        let segments = segments.max(3);
        let mut vertices = Vec::new();
        
        let half_height = height / 2.0;
        
        // Generate vertices for top and bottom circles
        let mut top_verts = Vec::new();
        let mut bottom_verts = Vec::new();
        
        for i in 0..segments {
            let angle = (i as f32) * 2.0 * std::f32::consts::PI / (segments as f32);
            let x = radius * angle.cos();
            let z = radius * angle.sin();
            
            // Color varies by height and angle
            let hue = angle / (2.0 * std::f32::consts::PI);
            let top_color = hsv_to_rgb(hue, 0.6, 1.0);
            let bottom_color = hsv_to_rgb(hue, 0.6, 0.7);
            
            top_verts.push(Vertex {
                position: [x, half_height, z],
                color: top_color,
            });
            
            bottom_verts.push(Vertex {
                position: [x, -half_height, z],
                color: bottom_color,
            });
        }
        
        // Create side faces (quads made of two triangles each)
        for i in 0..segments {
            let next_i = (i + 1) % segments;
            
            // First triangle of quad (counter-clockwise from outside)
            vertices.push(bottom_verts[i as usize]);
            vertices.push(top_verts[next_i as usize]);
            vertices.push(top_verts[i as usize]);
            
            // Second triangle of quad (counter-clockwise from outside)
            vertices.push(bottom_verts[i as usize]);
            vertices.push(bottom_verts[next_i as usize]);
            vertices.push(top_verts[next_i as usize]);
        }
        
        // Create top cap (triangular fan)
        let top_center = Vertex {
            position: [0.0, half_height, 0.0],
            color: [1.0, 0.8, 0.8], // Light red
        };
        
        for i in 0..segments {
            let next_i = (i + 1) % segments;
            vertices.push(top_center);
            vertices.push(top_verts[i as usize]); // Correct winding for upward normal
            vertices.push(top_verts[next_i as usize]);
        }
        
        // Create bottom cap (triangular fan)
        let bottom_center = Vertex {
            position: [0.0, -half_height, 0.0],
            color: [0.8, 0.8, 1.0], // Light blue
        };
        
        for i in 0..segments {
            let next_i = (i + 1) % segments;
            vertices.push(bottom_center);
            vertices.push(bottom_verts[next_i as usize]); // Reverse winding for downward normal
            vertices.push(bottom_verts[i as usize]);
        }
        
        Self { vertices, radius, height, segments }
    }

    /// Get the radius of the cylinder
    pub fn radius(&self) -> f32 {
        self.radius
    }

    /// Get the height of the cylinder
    pub fn height(&self) -> f32 {
        self.height
    }

    /// Get the number of segments around the cylinder
    pub fn segments(&self) -> u32 {
        self.segments
    }
}

impl VertexProvider for CylinderMesh {
    fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }
    
    fn vertex_count(&self) -> usize {
        self.vertices.len()
    }
    
    fn buffer_contents(&self) -> &[u8] {
        bytemuck::cast_slice(&self.vertices)
    }
}

impl Mesh for CylinderMesh {
    fn mesh_type(&self) -> &'static str {
        "Cylinder"
    }
    
    fn bounds(&self) -> (glam::Vec3, glam::Vec3) {
        let half_height = self.height / 2.0;
        (
            glam::Vec3::new(-self.radius, -half_height, -self.radius),
            glam::Vec3::new(self.radius, half_height, self.radius),
        )
    }
}

/// Cone mesh (3D cone with circular base)
#[derive(Debug, Clone)]
pub struct ConeMesh {
    vertices: Vec<Vertex>,
    radius: f32,
    height: f32,
    segments: u32,
}

impl ConeMesh {
    /// Create a new cone mesh with the given radius, height, and number of segments
    pub fn new(radius: f32, height: f32, segments: u32) -> Self {
        let segments = segments.max(3);
        let mut vertices = Vec::new();
        
        let half_height = height / 2.0;
        
        // Apex vertex (top of cone)
        let apex = Vertex {
            position: [0.0, half_height, 0.0],
            color: [1.0, 1.0, 0.0], // Yellow
        };
        
        // Generate base circle vertices
        let mut base_verts = Vec::new();
        for i in 0..segments {
            let angle = (i as f32) * 2.0 * std::f32::consts::PI / (segments as f32);
            let x = radius * angle.cos();
            let z = radius * angle.sin();
            
            let hue = angle / (2.0 * std::f32::consts::PI);
            let color = hsv_to_rgb(hue, 0.8, 0.9);
            
            base_verts.push(Vertex {
                position: [x, -half_height, z],
                color,
            });
        }
        
        // Create side faces (triangles from apex to base edge)
        for i in 0..segments {
            let next_i = (i + 1) % segments;
            
            vertices.push(apex);
            vertices.push(base_verts[i as usize]); // Correct winding for outward-facing triangles
            vertices.push(base_verts[next_i as usize]);
        }
        
        // Create base (triangular fan) - winding for downward-facing surface
        let base_center = Vertex {
            position: [0.0, -half_height, 0.0],
            color: [0.8, 0.8, 0.8], // Gray
        };
        
        for i in 0..segments {
            let next_i = (i + 1) % segments;
            vertices.push(base_center);
            vertices.push(base_verts[next_i as usize]); // Reversed winding for downward face
            vertices.push(base_verts[i as usize]);
        }
        
        Self { vertices, radius, height, segments }
    }

    /// Get the radius of the cone base
    pub fn radius(&self) -> f32 {
        self.radius
    }

    /// Get the height of the cone
    pub fn height(&self) -> f32 {
        self.height
    }

    /// Get the number of segments around the cone base
    pub fn segments(&self) -> u32 {
        self.segments
    }
}

impl VertexProvider for ConeMesh {
    fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }
    
    fn vertex_count(&self) -> usize {
        self.vertices.len()
    }
    
    fn buffer_contents(&self) -> &[u8] {
        bytemuck::cast_slice(&self.vertices)
    }
}

impl Mesh for ConeMesh {
    fn mesh_type(&self) -> &'static str {
        "Cone"
    }
    
    fn bounds(&self) -> (glam::Vec3, glam::Vec3) {
        let half_height = self.height / 2.0;
        (
            glam::Vec3::new(-self.radius, -half_height, -self.radius),
            glam::Vec3::new(self.radius, half_height, self.radius),
        )
    }
}

/// Sphere mesh (3D sphere using UV sphere generation)
#[derive(Debug, Clone)]
pub struct SphereMesh {
    vertices: Vec<Vertex>,
    radius: f32,
    latitude_segments: u32,
    longitude_segments: u32,
}

impl SphereMesh {
    /// Create a new sphere mesh with the given radius and subdivision counts
    pub fn new(radius: f32, latitude_segments: u32, longitude_segments: u32) -> Self {
        let lat_segs = latitude_segments.max(3);
        let lon_segs = longitude_segments.max(3);
        let mut vertices = Vec::new();
        
        // Generate sphere vertices using spherical coordinates
        let mut sphere_verts = Vec::new();
        
        for lat in 0..=lat_segs {
            let theta = (lat as f32) * std::f32::consts::PI / (lat_segs as f32);
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();
            
            for lon in 0..=lon_segs {
                let phi = (lon as f32) * 2.0 * std::f32::consts::PI / (lon_segs as f32);
                let sin_phi = phi.sin();
                let cos_phi = phi.cos();
                
                let x = radius * sin_theta * cos_phi;
                let y = radius * cos_theta;
                let z = radius * sin_theta * sin_phi;
                
                // Color based on position (colorful sphere)
                let color = [
                    (sin_theta * cos_phi + 1.0) * 0.5,
                    (cos_theta + 1.0) * 0.5,
                    (sin_theta * sin_phi + 1.0) * 0.5,
                ];
                
                sphere_verts.push(Vertex {
                    position: [x, y, z],
                    color,
                });
            }
        }
        
        // Generate triangles with proper winding for outward-facing normals
        for lat in 0..lat_segs {
            for lon in 0..lon_segs {
                // Calculate vertex indices for the current quad
                let i0 = lat * (lon_segs + 1) + lon;           // Bottom-left
                let i1 = lat * (lon_segs + 1) + (lon + 1);     // Bottom-right
                let i2 = (lat + 1) * (lon_segs + 1) + lon;     // Top-left
                let i3 = (lat + 1) * (lon_segs + 1) + (lon + 1); // Top-right
                
                // First triangle: bottom-left -> bottom-right -> top-left
                // This creates counter-clockwise winding when viewed from outside
                vertices.push(sphere_verts[i0 as usize]);
                vertices.push(sphere_verts[i1 as usize]);
                vertices.push(sphere_verts[i2 as usize]);
                
                // Second triangle: bottom-right -> top-right -> top-left
                // This also creates counter-clockwise winding when viewed from outside
                vertices.push(sphere_verts[i1 as usize]);
                vertices.push(sphere_verts[i3 as usize]);
                vertices.push(sphere_verts[i2 as usize]);
            }
        }
        
        Self { vertices, radius, latitude_segments: lat_segs, longitude_segments: lon_segs }
    }

    /// Get the radius of the sphere
    pub fn radius(&self) -> f32 {
        self.radius
    }

    /// Get the number of latitude segments
    pub fn latitude_segments(&self) -> u32 {
        self.latitude_segments
    }

    /// Get the number of longitude segments
    pub fn longitude_segments(&self) -> u32 {
        self.longitude_segments
    }
}

impl VertexProvider for SphereMesh {
    fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }
    
    fn vertex_count(&self) -> usize {
        self.vertices.len()
    }
    
    fn buffer_contents(&self) -> &[u8] {
        bytemuck::cast_slice(&self.vertices)
    }
}

impl Mesh for SphereMesh {
    fn mesh_type(&self) -> &'static str {
        "Sphere"
    }
    
    fn bounds(&self) -> (glam::Vec3, glam::Vec3) {
        (
            glam::Vec3::new(-self.radius, -self.radius, -self.radius),
            glam::Vec3::new(self.radius, self.radius, self.radius),
        )
    }
}

// Helper function to convert HSV to RGB
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [f32; 3] {
    let c = v * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = v - c;
    
    let (r, g, b) = if h < 1.0 / 6.0 {
        (c, x, 0.0)
    } else if h < 2.0 / 6.0 {
        (x, c, 0.0)
    } else if h < 3.0 / 6.0 {
        (0.0, c, x)
    } else if h < 4.0 / 6.0 {
        (0.0, x, c)
    } else if h < 5.0 / 6.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    
    [r + m, g + m, b + m]
}
