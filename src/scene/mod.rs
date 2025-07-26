//! Scene management and entity system
//! 
//! Provides a high-level interface for managing multiple renderable objects

use std::collections::HashMap;
use crate::renderable::{Renderable, Triangle, Quad, Cube, Circle, Cylinder, Cone, Sphere};

pub type EntityId = u32;

/// Manages a collection of renderable entities
/// Supports all primitive types: triangles, quads, cubes, circles, cylinders, cones, spheres, and tori
pub struct Scene {
    triangles: HashMap<EntityId, Triangle>,
    quads: HashMap<EntityId, Quad>,
    cubes: HashMap<EntityId, Cube>,
    circles: HashMap<EntityId, Circle>,
    cylinders: HashMap<EntityId, Cylinder>,
    cones: HashMap<EntityId, Cone>,
    spheres: HashMap<EntityId, Sphere>,
    next_id: EntityId,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            triangles: HashMap::new(),
            quads: HashMap::new(),
            cubes: HashMap::new(),
            circles: HashMap::new(),
            cylinders: HashMap::new(),
            cones: HashMap::new(),
            spheres: HashMap::new(),
            next_id: 0,
        }
    }
    
    /// Add a triangle to the scene and return its ID
    pub fn add_triangle(&mut self, triangle: Triangle) -> EntityId {
        let id = self.next_id;
        self.triangles.insert(id, triangle);
        self.next_id += 1;
        id
    }
    
    /// Add a quad to the scene and return its ID
    pub fn add_quad(&mut self, quad: Quad) -> EntityId {
        let id = self.next_id;
        self.quads.insert(id, quad);
        self.next_id += 1;
        id
    }
    
    /// Add a cube to the scene and return its ID
    pub fn add_cube(&mut self, cube: Cube) -> EntityId {
        let id = self.next_id;
        self.cubes.insert(id, cube);
        self.next_id += 1;
        id
    }
    
    /// Add a circle to the scene and return its ID
    pub fn add_circle(&mut self, circle: Circle) -> EntityId {
        let id = self.next_id;
        self.circles.insert(id, circle);
        self.next_id += 1;
        id
    }
    
    /// Add a cylinder to the scene and return its ID
    pub fn add_cylinder(&mut self, cylinder: Cylinder) -> EntityId {
        let id = self.next_id;
        self.cylinders.insert(id, cylinder);
        self.next_id += 1;
        id
    }
    
    /// Add a cone to the scene and return its ID
    pub fn add_cone(&mut self, cone: Cone) -> EntityId {
        let id = self.next_id;
        self.cones.insert(id, cone);
        self.next_id += 1;
        id
    }
    
    /// Add a sphere to the scene and return its ID
    pub fn add_sphere(&mut self, sphere: Sphere) -> EntityId {
        let id = self.next_id;
        self.spheres.insert(id, sphere);
        self.next_id += 1;
        id
    }
    
    /// Remove a triangle from the scene
    pub fn remove_triangle(&mut self, id: EntityId) -> Option<Triangle> {
        self.triangles.remove(&id)
    }
    
    /// Get a mutable reference to a triangle
    pub fn get_triangle_mut(&mut self, id: EntityId) -> Option<&mut Triangle> {
        self.triangles.get_mut(&id)
    }
    
    /// Update all entities in the scene
    pub fn update(&mut self, delta_time: f32) {
        for triangle in self.triangles.values_mut() {
            triangle.update(delta_time);
        }
        for quad in self.quads.values_mut() {
            quad.update(delta_time);
        }
        for cube in self.cubes.values_mut() {
            cube.update(delta_time);
        }
        for circle in self.circles.values_mut() {
            circle.update(delta_time);
        }
        for cylinder in self.cylinders.values_mut() {
            cylinder.update(delta_time);
        }
        for cone in self.cones.values_mut() {
            cone.update(delta_time);
        }
        for sphere in self.spheres.values_mut() {
            sphere.update(delta_time);
        }
    }
    
    /// Render all triangles in the scene using batch rendering
    pub fn render_triangles_batch<F>(&self, mut render_fn: F) -> Result<(), wgpu::SurfaceError>
    where
        F: FnMut(&[&Triangle]) -> Result<(), wgpu::SurfaceError>,
    {
        let triangles: Vec<&Triangle> = self.triangles.values().collect();
        log::debug!("Batch rendering {} triangles", triangles.len());
        render_fn(&triangles)
    }

    /// Render all quads in the scene using batch rendering
    pub fn render_quads_batch<F>(&self, mut render_fn: F) -> Result<(), wgpu::SurfaceError>
    where
        F: FnMut(&[&Quad]) -> Result<(), wgpu::SurfaceError>,
    {
        let quads: Vec<&Quad> = self.quads.values().collect();
        log::debug!("Batch rendering {} quads", quads.len());
        render_fn(&quads)
    }

    /// Render all cubes in the scene using batch rendering
    pub fn render_cubes_batch<F>(&self, mut render_fn: F) -> Result<(), wgpu::SurfaceError>
    where
        F: FnMut(&[&Cube]) -> Result<(), wgpu::SurfaceError>,
    {
        let cubes: Vec<&Cube> = self.cubes.values().collect();
        log::debug!("Batch rendering {} cubes", cubes.len());
        render_fn(&cubes)
    }

    /// Render all objects in the scene (triangles, quads, and cubes)
    pub fn render_all<F>(&self, mut render_fn: F) -> Result<(), wgpu::SurfaceError>
    where
        F: FnMut(&[&Triangle]) -> Result<(), wgpu::SurfaceError>
            + FnMut(&[&Quad]) -> Result<(), wgpu::SurfaceError>  
            + FnMut(&[&Cube]) -> Result<(), wgpu::SurfaceError>,
    {
        // Render each object type in separate batches
        self.render_triangles_batch(&mut render_fn)?;
        self.render_quads_batch(&mut render_fn)?;
        self.render_cubes_batch(&mut render_fn)?;
        Ok(())
    }

    /// Get all renderable objects as collections for debugging and unified rendering
    pub fn get_all_renderables(&self) -> (Vec<&Triangle>, Vec<&Quad>, Vec<&Cube>, Vec<&Circle>, Vec<&Cylinder>, Vec<&Cone>, Vec<&Sphere>) {
        let triangles: Vec<&Triangle> = self.triangles.values().collect();
        let quads: Vec<&Quad> = self.quads.values().collect();
        let cubes: Vec<&Cube> = self.cubes.values().collect();
        let circles: Vec<&Circle> = self.circles.values().collect();
        let cylinders: Vec<&Cylinder> = self.cylinders.values().collect();
        let cones: Vec<&Cone> = self.cones.values().collect();
        let spheres: Vec<&Sphere> = self.spheres.values().collect();
        (triangles, quads, cubes, circles, cylinders, cones, spheres)
    }

    /// Render all triangles with mutable access for dirty flag management
    pub fn render_triangles_batch_mut<F>(&mut self, mut render_fn: F) -> Result<(), wgpu::SurfaceError>
    where
        F: FnMut(&mut [&mut Triangle]) -> Result<(), wgpu::SurfaceError>,
    {
        let mut triangles: Vec<&mut Triangle> = self.triangles.values_mut().collect();
        log::debug!("Batch rendering {} triangles (mutable)", triangles.len());
        render_fn(&mut triangles)
    }
    
    /// Get the number of triangles in the scene
    pub fn triangle_count(&self) -> usize {
        self.triangles.len()
    }

    /// Get the number of quads in the scene
    pub fn quad_count(&self) -> usize {
        self.quads.len()
    }

    /// Get the number of cubes in the scene
    pub fn cube_count(&self) -> usize {
        self.cubes.len()
    }
    
    /// Get the number of circles in the scene
    pub fn circle_count(&self) -> usize {
        self.circles.len()
    }
    
    /// Get the number of cylinders in the scene
    pub fn cylinder_count(&self) -> usize {
        self.cylinders.len()
    }
    
    /// Get the number of cones in the scene
    pub fn cone_count(&self) -> usize {
        self.cones.len()
    }
    
    /// Get the number of spheres in the scene
    pub fn sphere_count(&self) -> usize {
        self.spheres.len()
    }
    
    /// Get the number of tori in the scene
    
    // === 3D Primitive Creation Functions ===
    
    /// Create a triangle primitive and add it to the scene
    pub fn create_triangle(&mut self, scale: f32) -> EntityId {
        let triangle = Triangle::with_scale(scale);
        self.add_triangle(triangle)
    }
    
    /// Create a triangle primitive at a specific position
    pub fn create_triangle_at(&mut self, scale: f32, position: glam::Vec3) -> EntityId {
        let mut triangle = Triangle::with_scale(scale);
        triangle.transform_set_position(position);
        self.add_triangle(triangle)
    }
    
    /// Create a triangle primitive with custom transform
    pub fn create_triangle_with_transform(&mut self, scale: f32, position: glam::Vec3, rotation: glam::Vec3) -> EntityId {
        let mut triangle = Triangle::with_scale(scale);
        triangle.transform_set_position(position);
        triangle.transform_rotate_degrees(rotation.x, rotation.y, rotation.z);
        self.add_triangle(triangle)
    }

    /// Create a quad primitive and add it to the scene
    pub fn create_quad(&mut self, size: f32) -> EntityId {
        let quad = Quad::with_size(size, size);  // Square quad
        self.add_quad(quad)
    }
    
    /// Create a quad primitive at a specific position
    pub fn create_quad_at(&mut self, size: f32, position: glam::Vec3) -> EntityId {
        let mut quad = Quad::with_size(size, size);  // Square quad
        quad.transform_set_position(position);
        self.add_quad(quad)
    }
    
    /// Create a quad primitive with custom transform
    pub fn create_quad_with_transform(&mut self, size: f32, position: glam::Vec3, rotation: glam::Vec3) -> EntityId {
        let mut quad = Quad::with_size(size, size);  // Square quad
        quad.transform_set_position(position);
        quad.transform_rotate_degrees(rotation.x, rotation.y, rotation.z);
        self.add_quad(quad)
    }

    /// Create a cube primitive and add it to the scene
    pub fn create_cube(&mut self, size: f32) -> EntityId {
        let cube = Cube::with_size(size);
        self.add_cube(cube)
    }
    
    /// Create a cube primitive at a specific position
    pub fn create_cube_at(&mut self, size: f32, position: glam::Vec3) -> EntityId {
        let mut cube = Cube::with_size(size);
        cube.transform_set_position(position);
        self.add_cube(cube)
    }
    
    /// Create a cube primitive with custom transform
    pub fn create_cube_with_transform(&mut self, size: f32, position: glam::Vec3, rotation: glam::Vec3) -> EntityId {
        let mut cube = Cube::with_size(size);
        cube.transform_set_position(position);
        cube.transform_rotate_degrees(rotation.x, rotation.y, rotation.z);
        self.add_cube(cube)
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    #[test]
    fn test_scene_creation() {
        let scene = Scene::new();
        assert_eq!(scene.triangle_count(), 0);
        assert_eq!(scene.quad_count(), 0);
        assert_eq!(scene.cube_count(), 0);
    }

    #[test]
    fn test_scene_default() {
        let scene = Scene::default();
        assert_eq!(scene.triangle_count(), 0);
        assert_eq!(scene.quad_count(), 0);
        assert_eq!(scene.cube_count(), 0);
    }

    #[test]
    fn test_add_triangle() {
        let mut scene = Scene::new();
        let triangle = Triangle::new();
        let id = scene.add_triangle(triangle);
        
        assert_eq!(id, 0);
        assert_eq!(scene.triangle_count(), 1);
        assert_eq!(scene.quad_count(), 0);
        assert_eq!(scene.cube_count(), 0);
    }

    #[test]
    fn test_add_quad() {
        let mut scene = Scene::new();
        let quad = Quad::with_size(1.0, 1.0);
        let id = scene.add_quad(quad);
        
        assert_eq!(id, 0);
        assert_eq!(scene.triangle_count(), 0);
        assert_eq!(scene.quad_count(), 1);
        assert_eq!(scene.cube_count(), 0);
    }

    #[test]
    fn test_add_cube() {
        let mut scene = Scene::new();
        let cube = Cube::with_size(1.0);
        let id = scene.add_cube(cube);
        
        assert_eq!(id, 0);
        assert_eq!(scene.triangle_count(), 0);
        assert_eq!(scene.quad_count(), 0);
        assert_eq!(scene.cube_count(), 1);
    }

    #[test]
    fn test_entity_id_sequencing() {
        let mut scene = Scene::new();
        
        let triangle_id = scene.add_triangle(Triangle::new());
        let quad_id = scene.add_quad(Quad::with_size(1.0, 1.0));
        let cube_id = scene.add_cube(Cube::with_size(1.0));
        
        assert_eq!(triangle_id, 0);
        assert_eq!(quad_id, 1);
        assert_eq!(cube_id, 2);
    }

    #[test]
    fn test_create_triangle_methods() {
        let mut scene = Scene::new();
        
        // Test basic triangle creation
        let _id1 = scene.create_triangle(2.0);
        assert_eq!(scene.triangle_count(), 1);
        
        // Test triangle at specific position
        let position = Vec3::new(5.0, 10.0, 15.0);
        let _id2 = scene.create_triangle_at(1.5, position);
        assert_eq!(scene.triangle_count(), 2);
        
        // Test triangle with custom transform
        let rotation = Vec3::new(45.0, 90.0, 180.0);
        let _id3 = scene.create_triangle_with_transform(1.0, position, rotation);
        assert_eq!(scene.triangle_count(), 3);
    }

    #[test]
    fn test_basic_scene_operations() {
        let mut scene = Scene::new();
        
        // Test adding objects
        let triangle_id = scene.add_triangle(Triangle::new());
        let quad_id = scene.add_quad(Quad::with_size(1.0, 1.0));
        let cube_id = scene.add_cube(Cube::with_size(1.0));
        
        // Test counting
        assert_eq!(scene.triangle_count(), 1);
        assert_eq!(scene.quad_count(), 1);
        assert_eq!(scene.cube_count(), 1);
        
        // Test getting all renderables
        let (triangles, quads, cubes, circles, cylinders, cones, spheres) = scene.get_all_renderables();
        assert_eq!(triangles.len(), 1);
        assert_eq!(quads.len(), 1);
        assert_eq!(cubes.len(), 1);
        assert_eq!(circles.len(), 0);
        assert_eq!(cylinders.len(), 0);
        assert_eq!(cones.len(), 0);
        assert_eq!(spheres.len(), 0);
        
        // Test removing objects (only triangle removal is implemented)
        assert!(scene.remove_triangle(triangle_id).is_some());
        
        assert_eq!(scene.triangle_count(), 0);
        assert_eq!(scene.quad_count(), 1); // Quad still exists
        assert_eq!(scene.cube_count(), 1); // Cube still exists
    }

    #[test]
    fn test_update_all_entities() {
        let mut scene = Scene::new();
        
        scene.add_triangle(Triangle::new());
        scene.add_quad(Quad::with_size(1.0, 1.0));
        scene.add_cube(Cube::with_size(1.0));
        
        // This should not panic - validates that update() can be called
        scene.update(0.016); // 60 FPS delta time
    }
}
