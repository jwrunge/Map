//! Vertex buffer cache for static geometry optimization
//!
//! This module provides caching for vertex buffers to avoid recreating
//! the same geometry data every frame.

use crate::renderable::VertexProvider;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::time::Instant;
use wgpu::util::DeviceExt;

/// Hash key for identifying identical vertex data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct VertexDataHash(u64);

/// Cached vertex buffer with metadata
pub struct CachedVertexBuffer {
    pub buffer: wgpu::Buffer,
    pub vertex_count: u32,
    last_used: Instant,
}

/// High-performance vertex buffer cache
pub struct VertexBufferCache {
    cache: HashMap<VertexDataHash, CachedVertexBuffer>,
    max_age_seconds: u64,
}

impl VertexBufferCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            max_age_seconds: 30, // Keep buffers for 30 seconds
        }
    }

    /// Clean up old cache entries to prevent memory leaks
    pub fn cleanup_old_entries(&mut self) {
        let now = Instant::now();
        self.cache.retain(|_hash, cached| {
            now.duration_since(cached.last_used).as_secs() < self.max_age_seconds
        });
    }

    /// Get cache statistics
    pub fn stats(&self) -> (usize, usize) {
        let total_entries = self.cache.len();
        let total_vertices: usize = self.cache.values().map(|c| c.vertex_count as usize).sum();
        (total_entries, total_vertices)
    }

    /// Process multiple renderables and return their vertex buffers
    /// This avoids borrowing issues by handling all objects in a single method call
    pub fn get_or_create_multiple_buffers<T: VertexProvider>(
        &mut self,
        renderables: &[T],
        device: &wgpu::Device,
    ) -> Vec<(&wgpu::Buffer, u32)> {
        let mut result = Vec::new();
        
        for renderable in renderables.iter() {
            let contents = renderable.buffer_contents();
            let hash = Self::hash_vertex_data(contents);

            // Check if we already have this buffer cached
            if let Some(cached) = self.cache.get_mut(&hash) {
                // Update the last used time
                cached.last_used = Instant::now();
            } else {
                // Create new buffer
                let vertex_count = renderable.vertex_count() as u32;
                let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Cached Vertex Buffer"),
                    contents,
                    usage: wgpu::BufferUsages::VERTEX,
                });

                let cached_buffer = CachedVertexBuffer {
                    buffer,
                    vertex_count,
                    last_used: Instant::now(),
                };

                self.cache.insert(hash, cached_buffer);
            }
        }
        
        // Now collect all the references (this works because we're not mutating anymore)
        for renderable in renderables.iter() {
            let contents = renderable.buffer_contents();
            let hash = Self::hash_vertex_data(contents);
            
            if let Some(cached) = self.cache.get(&hash) {
                result.push((&cached.buffer, cached.vertex_count));
            }
        }
        
        result
    }

    /// Hash vertex data for cache key
    fn hash_vertex_data(data: &[u8]) -> VertexDataHash {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        data.hash(&mut hasher);
        VertexDataHash(hasher.finish())
    }
}

impl Default for VertexBufferCache {
    fn default() -> Self {
        Self::new()
    }
}
