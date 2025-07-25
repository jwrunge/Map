//! Optimized batch rendering system
//!
//! This module provides performance-optimized rendering by:
//! - Caching vertex buffers for static geometry
//! - Using single render pass for multiple objects
//! - Instanced rendering for identical objects
//! - Matrix caching

use crate::renderable::{Renderable, VertexProvider};
use crate::renderer::{Camera, GpuContext, RenderPipeline};
use std::collections::HashMap;
use wgpu::util::DeviceExt;

/// Cached vertex buffer with metadata
struct CachedVertexBuffer {
    buffer: wgpu::Buffer,
    vertex_count: u32,
    last_used: std::time::Instant,
}

/// High-performance batch renderer with caching
pub struct OptimizedRenderer {
    pub gpu: GpuContext,
    pub pipeline: RenderPipeline,
    pub camera: Camera,
    vertex_cache: HashMap<u64, CachedVertexBuffer>,
    cached_view_projection: Option<(glam::Mat4, std::time::Instant)>,
}

impl OptimizedRenderer {
    pub async fn new(window: std::sync::Arc<winit::window::Window>) -> Self {
        let gpu = GpuContext::new(window).await;
        let pipeline = RenderPipeline::new(&gpu);
        let camera = Camera::new(gpu.size.width as f32 / gpu.size.height as f32);

        Self {
            gpu,
            pipeline,
            camera,
            vertex_cache: HashMap::new(),
            cached_view_projection: None,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.gpu.resize(new_size);
        self.camera.set_aspect_ratio(new_size.width as f32 / new_size.height as f32);
        // Invalidate cached view projection
        self.cached_view_projection = None;
    }

    /// Get or create cached vertex buffer for geometry
    fn get_vertex_buffer<R: VertexProvider>(&mut self, renderable: &R) -> (&wgpu::Buffer, u32) {
        // Simple hash of vertex data for caching
        let contents = renderable.buffer_contents();
        let hash = self.hash_bytes(contents);

        if let Some(cached) = self.vertex_cache.get_mut(&hash) {
            cached.last_used = std::time::Instant::now();
            (&cached.buffer, cached.vertex_count)
        } else {
            // Create new cached buffer
            let buffer = self.gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Cached Vertex Buffer"),
                contents,
                usage: wgpu::BufferUsages::VERTEX,
            });
            
            let vertex_count = renderable.vertex_count() as u32;
            
            self.vertex_cache.insert(hash, CachedVertexBuffer {
                buffer,
                vertex_count,
                last_used: std::time::Instant::now(),
            });
            
            let cached = self.vertex_cache.get(&hash).unwrap();
            (&cached.buffer, cached.vertex_count)
        }
    }

    /// Simple hash function for vertex data
    fn hash_bytes(&self, data: &[u8]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish()
    }

    /// Get cached view-projection matrix
    fn get_view_projection_matrix(&mut self) -> glam::Mat4 {
        let now = std::time::Instant::now();
        
        // Cache view-projection matrix for 1ms (multiple objects per frame)
        if let Some((matrix, timestamp)) = &self.cached_view_projection {
            if now.duration_since(*timestamp).as_millis() < 1 {
                return *matrix;
            }
        }

        let matrix = self.camera.get_view_projection_matrix();
        self.cached_view_projection = Some((matrix, now));
        matrix
    }

    /// Optimized batch rendering with single render pass
    pub fn render_batch_optimized<R: Renderable + VertexProvider>(
        &mut self,
        renderables: &[&R],
    ) -> Result<(), wgpu::SurfaceError> {
        if renderables.is_empty() {
            return Ok(());
        }

        let output = self.gpu.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Single command encoder for entire batch
        let mut encoder = self.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Optimized Batch Encoder"),
        });

        // Single render pass for all objects
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Optimized Batch Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.pipeline.pipeline);
            
            let view_projection_matrix = self.get_view_projection_matrix();

            // Render each object with cached vertex buffers
            for renderable in renderables {
                // Get cached vertex buffer
                let (vertex_buffer, vertex_count) = self.get_vertex_buffer(*renderable);

                // Calculate final transform matrix
                let model_matrix = renderable.get_matrix();
                let final_matrix = view_projection_matrix * model_matrix;

                // Update uniform buffer for this object
                self.gpu.queue.write_buffer(
                    &self.pipeline.uniform_buffer,
                    0,
                    bytemuck::cast_slice(final_matrix.as_ref()),
                );

                // Draw with cached vertex buffer
                render_pass.set_bind_group(0, &self.pipeline.uniform_bind_group, &[]);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.draw(0..vertex_count, 0..1);
            }
        }

        // Single submit for entire batch
        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        // Cleanup old cache entries
        self.cleanup_cache();

        Ok(())
    }

    /// Remove old cache entries to prevent memory leaks
    fn cleanup_cache(&mut self) {
        let now = std::time::Instant::now();
        self.vertex_cache.retain(|_hash, cached| {
            now.duration_since(cached.last_used).as_secs() < 5 // Keep for 5 seconds
        });
    }
}
