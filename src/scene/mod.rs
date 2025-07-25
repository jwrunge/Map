//! Scene management and entity system
//! 
//! Provides a high-level interface for managing multiple renderable objects

use std::collections::HashMap;
use crate::renderable::{Renderable, Triangle};

pub type EntityId = u32;

/// Manages a collection of renderable entities
/// For now, this is simplified to work with concrete types
pub struct Scene {
    triangles: HashMap<EntityId, Triangle>,
    next_id: EntityId,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            triangles: HashMap::new(),
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
    }
    
    /// Render all triangles in the scene
    pub fn render_triangles<F>(&self, mut render_fn: F) -> Result<(), wgpu::SurfaceError>
    where
        F: FnMut(&Triangle) -> Result<(), wgpu::SurfaceError>,
    {
        for triangle in self.triangles.values() {
            render_fn(triangle)?;
        }
        Ok(())
    }
    
    /// Get the number of triangles in the scene
    pub fn triangle_count(&self) -> usize {
        self.triangles.len()
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}
