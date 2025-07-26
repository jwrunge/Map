//! Renderable objects and graphics primitives
//!
//! This module provides the core traits and implementations for objects
//! that can be rendered to the screen.

use glam;
use web_time::Instant;
use crate::renderer::config::CullingMode;

pub mod mesh;
mod transforms;
pub mod vertex;

pub use mesh::{CubeMesh, Mesh, QuadMesh, TriangleMesh, CircleMesh, CylinderMesh, ConeMesh, SphereMesh};
pub use transforms::Transform;
pub use vertex::{Vertex, VertexProvider};

/// Quad object (rectangle made of two triangles)
#[derive(Debug, Clone)]
pub struct Quad {
    mesh: QuadMesh,
    transform: Transform,
    is_dirty: bool,
    culling_mode: CullingMode,
}

impl Quad {
    /// Create a new quad with the given dimensions
    pub fn with_size(width: f32, height: f32) -> Self {
        Self {
            mesh: QuadMesh::new(width, height),
            transform: Transform::new(),
            is_dirty: true,
            culling_mode: CullingMode::None, // 2D objects default to no culling
        }
    }

    /// Get access to the mesh for rendering
    pub fn mesh(&self) -> &QuadMesh {
        &self.mesh
    }

    /// Get access to the transform
    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    /// Get the current culling mode
    pub fn get_culling_mode(&self) -> CullingMode {
        self.culling_mode
    }

    /// Set the culling mode
    pub fn set_culling_mode(&mut self, mode: CullingMode) {
        self.culling_mode = mode;
    }
}

/// Cube object (3D cube made of 12 triangles)
#[derive(Debug, Clone)]
pub struct Cube {
    mesh: CubeMesh,
    transform: Transform,
    is_dirty: bool,
    culling_mode: CullingMode,
}

impl Cube {
    /// Create a new cube with the given size
    pub fn with_size(size: f32) -> Self {
        Self {
            mesh: CubeMesh::new(size),
            transform: Transform::new(),
            is_dirty: true,
            culling_mode: CullingMode::BackfaceCulling, // 3D objects benefit from backface culling
        }
    }

    /// Get access to the mesh for rendering
    pub fn mesh(&self) -> &CubeMesh {
        &self.mesh
    }

    /// Get access to the transform
    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    /// Get the current culling mode
    pub fn get_culling_mode(&self) -> CullingMode {
        self.culling_mode
    }

    /// Set the culling mode
    pub fn set_culling_mode(&mut self, mode: CullingMode) {
        self.culling_mode = mode;
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
        self.transform.get_matrix()
    }

    fn get_matrix_cached(&mut self) -> glam::Mat4 {
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

    fn get_culling_mode(&self) -> CullingMode {
        self.culling_mode
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
        self.transform.get_matrix()
    }

    fn get_matrix_cached(&mut self) -> glam::Mat4 {
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

    fn get_culling_mode(&self) -> CullingMode {
        self.culling_mode
    }
}

// Implement VertexProvider for Cube
impl VertexProvider for Cube {
    fn vertices(&self) -> &[Vertex] {
        self.mesh.vertices()
    }
}

/// Circle object (2D circle made of triangular segments)
#[derive(Debug, Clone)]
pub struct Circle {
    mesh: CircleMesh,
    transform: Transform,
    is_dirty: bool,
    culling_mode: CullingMode,
    start_time: Instant,
}

impl Circle {
    /// Create a new circle with the given radius and number of segments
    pub fn new(radius: f32, segments: u32) -> Self {
        Self {
            mesh: CircleMesh::new(radius, segments),
            transform: Transform::new(),
            is_dirty: true,
            culling_mode: CullingMode::None, // 2D objects default to no culling
            start_time: Instant::now(),
        }
    }

    /// Get access to the mesh for rendering
    pub fn mesh(&self) -> &CircleMesh {
        &self.mesh
    }

    /// Get access to the transform
    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    /// Get the current culling mode
    pub fn get_culling_mode(&self) -> CullingMode {
        self.culling_mode
    }

    /// Set the culling mode
    pub fn set_culling_mode(&mut self, mode: CullingMode) {
        self.culling_mode = mode;
    }
}

impl Renderable for Circle {
    fn get_transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }

    fn get_transform(&self) -> &Transform {
        &self.transform
    }

    fn get_matrix(&self) -> glam::Mat4 {
        self.transform.get_matrix()
    }

    fn get_matrix_cached(&mut self) -> glam::Mat4 {
        self.transform.to_matrix()
    }

    fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    fn set_dirty(&mut self, dirty: bool) {
        self.is_dirty = dirty;
    }

    fn update(&mut self, delta: f32) {
        // Circle gently pulses with scale and rotates
        let elapsed_time = self.start_time.elapsed().as_secs_f32();
        let pulse = 1.0 + 0.1 * (elapsed_time * 2.0).sin();
        
        // Set absolute scale instead of accumulating
        self.transform.set_scale(glam::Vec3::new(pulse, pulse, 1.0));
        self.transform_rotate_degrees(0.0, 0.0, 30.0 * delta);
        self.set_dirty(true);
    }

    fn get_culling_mode(&self) -> CullingMode {
        self.culling_mode
    }
}

impl VertexProvider for Circle {
    fn vertices(&self) -> &[Vertex] {
        self.mesh.vertices()
    }
}

/// Cylinder object (3D cylinder with circular cross-section)
#[derive(Debug, Clone)]
pub struct Cylinder {
    mesh: CylinderMesh,
    transform: Transform,
    is_dirty: bool,
    culling_mode: CullingMode,
}

impl Cylinder {
    /// Create a new cylinder with the given radius, height, and number of segments
    pub fn new(radius: f32, height: f32, segments: u32) -> Self {
        Self {
            mesh: CylinderMesh::new(radius, height, segments),
            transform: Transform::new(),
            is_dirty: true,
            culling_mode: CullingMode::BackfaceCulling, // 3D objects benefit from backface culling
        }
    }

    /// Get access to the mesh for rendering
    pub fn mesh(&self) -> &CylinderMesh {
        &self.mesh
    }

    /// Get access to the transform
    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    /// Get the current culling mode
    pub fn get_culling_mode(&self) -> CullingMode {
        self.culling_mode
    }

    /// Set the culling mode
    pub fn set_culling_mode(&mut self, mode: CullingMode) {
        self.culling_mode = mode;
    }
}

impl Renderable for Cylinder {
    fn get_transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }

    fn get_transform(&self) -> &Transform {
        &self.transform
    }

    fn get_matrix(&self) -> glam::Mat4 {
        self.transform.get_matrix()
    }

    fn get_matrix_cached(&mut self) -> glam::Mat4 {
        self.transform.to_matrix()
    }

    fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    fn set_dirty(&mut self, dirty: bool) {
        self.is_dirty = dirty;
    }

    fn update(&mut self, delta: f32) {
        // Cylinder rotates around Y axis like a rolling log
        self.transform_rotate_degrees(45.0 * delta, 45.0 * delta, 0.0);
    }

    fn get_culling_mode(&self) -> CullingMode {
        self.culling_mode
    }
}

impl VertexProvider for Cylinder {
    fn vertices(&self) -> &[Vertex] {
        self.mesh.vertices()
    }
}

/// Cone object (3D cone with circular base)
#[derive(Debug, Clone)]
pub struct Cone {
    mesh: ConeMesh,
    transform: Transform,
    is_dirty: bool,
    culling_mode: CullingMode,
}

impl Cone {
    /// Create a new cone with the given radius, height, and number of segments
    pub fn new(radius: f32, height: f32, segments: u32) -> Self {
        Self {
            mesh: ConeMesh::new(radius, height, segments),
            transform: Transform::new(),
            is_dirty: true,
            culling_mode: CullingMode::BackfaceCulling, // 3D objects benefit from backface culling
        }
    }

    /// Get access to the mesh for rendering
    pub fn mesh(&self) -> &ConeMesh {
        &self.mesh
    }

    /// Get access to the transform
    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    /// Get the current culling mode
    pub fn get_culling_mode(&self) -> CullingMode {
        self.culling_mode
    }

    /// Set the culling mode
    pub fn set_culling_mode(&mut self, mode: CullingMode) {
        self.culling_mode = mode;
    }
}

impl Renderable for Cone {
    fn get_transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }

    fn get_transform(&self) -> &Transform {
        &self.transform
    }

    fn get_matrix(&self) -> glam::Mat4 {
        self.transform.get_matrix()
    }

    fn get_matrix_cached(&mut self) -> glam::Mat4 {
        self.transform.to_matrix()
    }

    fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    fn set_dirty(&mut self, dirty: bool) {
        self.is_dirty = dirty;
    }

    fn update(&mut self, delta: f32) {
        // Cone wobbles by rotating around X and Y axes
        self.transform_rotate_degrees(20.0 * delta, 15.0 * delta, 0.0);
    }

    fn get_culling_mode(&self) -> CullingMode {
        self.culling_mode
    }
}

impl VertexProvider for Cone {
    fn vertices(&self) -> &[Vertex] {
        self.mesh.vertices()
    }
}

/// Sphere object (3D sphere using UV sphere generation)
#[derive(Debug, Clone)]
pub struct Sphere {
    mesh: SphereMesh,
    transform: Transform,
    is_dirty: bool,
    culling_mode: CullingMode,
}

impl Sphere {
    /// Create a new sphere with the given radius and subdivision counts
    pub fn new(radius: f32, latitude_segments: u32, longitude_segments: u32) -> Self {
        Self {
            mesh: SphereMesh::new(radius, latitude_segments, longitude_segments),
            transform: Transform::new(),
            is_dirty: true,
            culling_mode: CullingMode::BackfaceCulling, // 3D objects benefit from backface culling
        }
    }

    /// Get access to the mesh for rendering
    pub fn mesh(&self) -> &SphereMesh {
        &self.mesh
    }

    /// Get access to the transform
    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    /// Get the current culling mode
    pub fn get_culling_mode(&self) -> CullingMode {
        self.culling_mode
    }

    /// Set the culling mode
    pub fn set_culling_mode(&mut self, mode: CullingMode) {
        self.culling_mode = mode;
    }
}

impl Renderable for Sphere {
    fn get_transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }

    fn get_transform(&self) -> &Transform {
        &self.transform
    }

    fn get_matrix(&self) -> glam::Mat4 {
        self.transform.get_matrix()
    }

    fn get_matrix_cached(&mut self) -> glam::Mat4 {
        self.transform.to_matrix()
    }

    fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    fn set_dirty(&mut self, dirty: bool) {
        self.is_dirty = dirty;
    }

    fn update(&mut self, delta: f32) {
        // Sphere rotates smoothly on multiple axes for a nice orbital effect
        self.transform_rotate_degrees(10.0 * delta, 25.0 * delta, 5.0 * delta);
    }

    fn get_culling_mode(&self) -> CullingMode {
        self.culling_mode
    }
}

impl VertexProvider for Sphere {
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
    fn get_matrix_cached(&mut self) -> glam::Mat4;
    fn get_culling_mode(&self) -> CullingMode;

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

#[derive(Debug, Clone)]
pub struct Triangle {
    is_dirty: bool,
    vertices: [Vertex; 3],
    transform: Transform,
    culling_mode: CullingMode,
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
            culling_mode: CullingMode::None, // 2D triangles should render both sides
        }
    }

    /// Get access to the vertices for rendering
    pub fn vertices(&self) -> &[Vertex; 3] {
        &self.vertices
    }

    /// Get access to the transform
    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    /// Get the current culling mode
    pub fn get_culling_mode(&self) -> CullingMode {
        self.culling_mode
    }

    /// Set the culling mode
    pub fn set_culling_mode(&mut self, mode: CullingMode) {
        self.culling_mode = mode;
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
        self.transform.get_matrix()
    }

    fn get_matrix_cached(&mut self) -> glam::Mat4 {
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

    fn get_culling_mode(&self) -> CullingMode {
        self.culling_mode
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

    fn get_matrix_cached(&mut self) -> glam::Mat4 {
        // Cannot mutate through & reference - fall back to uncached
        (**self).get_matrix()
    }

    fn get_culling_mode(&self) -> CullingMode {
        (**self).get_culling_mode()
    }
}

impl<T: VertexProvider> VertexProvider for &T {
    fn vertices(&self) -> &[Vertex] {
        (**self).vertices()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    #[test]
    fn test_triangle_creation() {
        let triangle = Triangle::new();
        assert_eq!(triangle.vertices().len(), 3);
    }

    #[test]
    fn test_triangle_with_scale() {
        let triangle = Triangle::with_scale(2.0);
        assert_eq!(triangle.vertices().len(), 3);
    }

    #[test]
    fn test_quad_creation() {
        let quad = Quad::with_size(1.0, 2.0);
        assert_eq!(quad.vertices().len(), 6); // 2 triangles
    }

    #[test]
    fn test_cube_creation() {
        let cube = Cube::with_size(1.0);
        assert_eq!(cube.vertices().len(), 36); // 12 triangles
    }

    #[test]
    fn test_circle_creation() {
        let circle = Circle::new(1.0, 8);
        assert_eq!(circle.vertices().len(), 24); // 8 triangles (3 vertices each)
    }

    #[test]
    fn test_cylinder_creation() {
        let cylinder = Cylinder::new(1.0, 2.0, 6);
        // 6 side quads (12 triangles) + 2 caps (6 triangles each) = 24 triangles = 72 vertices
        assert_eq!(cylinder.vertices().len(), 72);
    }

    #[test]
    fn test_cone_creation() {
        let cone = Cone::new(1.0, 2.0, 6);
        // 6 side triangles + 6 base triangles = 12 triangles = 36 vertices
        assert_eq!(cone.vertices().len(), 36);
    }

    #[test]
    fn test_sphere_creation() {
        let sphere = Sphere::new(1.0, 4, 8);
        // 4 latitude segments * 8 longitude segments * 2 triangles per quad = 64 triangles = 192 vertices
        assert_eq!(sphere.vertices().len(), 192);
    }

    #[test]
    fn test_culling_modes() {
        let triangle = Triangle::new();
        let quad = Quad::with_size(1.0, 1.0);
        let cube = Cube::with_size(1.0);
        let circle = Circle::new(1.0, 8);
        let cylinder = Cylinder::new(1.0, 2.0, 8);
        let cone = Cone::new(1.0, 2.0, 8);
        let sphere = Sphere::new(1.0, 8, 16);
        
        // 2D objects should have no culling
        assert_eq!(triangle.get_culling_mode(), CullingMode::None);
        assert_eq!(quad.get_culling_mode(), CullingMode::None);
        assert_eq!(circle.get_culling_mode(), CullingMode::None);
        
        // 3D objects should have backface culling
        assert_eq!(cube.get_culling_mode(), CullingMode::BackfaceCulling);
        assert_eq!(cylinder.get_culling_mode(), CullingMode::BackfaceCulling);
        assert_eq!(cone.get_culling_mode(), CullingMode::BackfaceCulling);
        assert_eq!(sphere.get_culling_mode(), CullingMode::BackfaceCulling);
    }

    #[test]
    fn test_renderable_transform() {
        let mut triangle = Triangle::new();
        let initial_pos = triangle.transform.position;
        
        // Test position setting
        let new_pos = Vec3::new(1.0, 2.0, 3.0);
        triangle.transform_set_position(new_pos);
        assert_eq!(triangle.transform.position, new_pos);
        assert_ne!(triangle.transform.position, initial_pos);
        
        // Test that transform doesn't break vertex generation
        assert_eq!(triangle.vertices().len(), 3);
    }

    #[test]
    fn test_update_method() {
        let mut triangle = Triangle::new();
        let mut quad = Quad::with_size(1.0, 1.0);
        let mut cube = Cube::with_size(1.0);
        
        // Should not panic
        triangle.update(0.016);
        quad.update(0.016);
        cube.update(0.016);
    }

    #[test]
    fn test_different_culling_groups() {
        let triangle = Triangle::new();
        let cube = Cube::with_size(1.0);
        
        // Triangles and cubes should have different culling modes
        assert_ne!(triangle.get_culling_mode(), cube.get_culling_mode());
        
        // This validates the grouping logic that was causing render issues
        assert_eq!(triangle.get_culling_mode(), CullingMode::None);
        assert_eq!(cube.get_culling_mode(), CullingMode::BackfaceCulling);
    }
}


