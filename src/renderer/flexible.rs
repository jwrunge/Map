//! Flexible rendering system supporting both individual and instanced drawing
//!
//! This module provides a unified interface for rendering that can optimize
//! automatically while preserving the ability to render objects individually.

use crate::renderable::{Renderable, VertexProvider};
use crate::renderer::{Camera, GpuContext, RenderPipeline};
use crate::renderer::vertex_cache::{VertexBufferCache, CachedVertexBuffer};
use std::collections::HashMap;
use wgpu::util::DeviceExt;

/// Rendering mode selection
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderMode {
    /// Render each object individually (current behavior)
    Individual,
    /// Group identical objects and render as instances (optimization)
    Instanced,
    /// Automatically choose best mode based on scene composition
    Auto,
}

/// Instance data for batched rendering
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    // 4x4 transform matrix stored as 4 Vec4s
    pub transform: [[f32; 4]; 4],
}

/// Smart renderer that can switch between individual and instanced rendering
pub struct FlexibleRenderer {
    pub gpu: GpuContext,
    pub pipeline: RenderPipeline,
    pub camera: Camera,
    vertex_cache: VertexBufferCache,
    cached_view_projection: Option<(glam::Mat4, std::time::Instant)>,
    render_mode: RenderMode,
}

impl FlexibleRenderer {
    pub async fn new(window: std::sync::Arc<winit::window::Window>) -> Self {
        let gpu = GpuContext::new(window).await;
        let pipeline = RenderPipeline::new(&gpu);
        let camera = Camera::new(gpu.size.width as f32 / gpu.size.height as f32);

        Self {
            gpu,
            pipeline,
            camera,
            vertex_cache: VertexBufferCache::new(),
            cached_view_projection: None,
            render_mode: RenderMode::Auto,
        }
    }

    pub fn set_render_mode(&mut self, mode: RenderMode) {
        self.render_mode = mode;
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.gpu.resize(new_size);
        self.camera.set_aspect_ratio(new_size.width as f32 / new_size.height as f32);
        self.cached_view_projection = None;
    }

    /// Get cached view-projection matrix (optimization)
    fn get_view_projection_matrix(&mut self) -> glam::Mat4 {
        let now = std::time::Instant::now();
        
        if let Some((matrix, timestamp)) = &self.cached_view_projection {
            if now.duration_since(*timestamp).as_millis() < 16 { // Cache for ~1 frame at 60fps
                return *matrix;
            }
        }

        let matrix = self.camera.get_view_projection_matrix();
        self.cached_view_projection = Some((matrix, now));
        matrix
    }

    /// Render objects with automatic mode selection
    pub fn render_batch_flexible<R: Renderable + VertexProvider>(
        &mut self,
        renderables: &[&R],
    ) -> Result<(), wgpu::SurfaceError> {
        if renderables.is_empty() {
            return Ok(());
        }

        // Choose rendering mode
        let mode = match self.render_mode {
            RenderMode::Individual => RenderMode::Individual,
            RenderMode::Instanced => RenderMode::Instanced,
            RenderMode::Auto => {
                // Auto-select based on object count and similarity
                if renderables.len() <= 2 {
                    RenderMode::Individual // Few objects, individual is fine
                } else {
                    // Check if objects have identical geometry (could be instanced)
                    let first_vertices = renderables[0].buffer_contents();
                    let all_same = renderables.iter().skip(1).all(|r| r.buffer_contents() == first_vertices);
                    
                    if all_same {
                        RenderMode::Instanced // Identical geometry, use instancing
                    } else {
                        RenderMode::Individual // Different geometry, use individual
                    }
                }
            }
        };

        match mode {
            RenderMode::Individual => self.render_individual(renderables),
            RenderMode::Instanced => {
                // For now, fall back to individual if instancing isn't fully implemented
                // This ensures we don't break existing functionality
                self.render_individual(renderables)
            }
            RenderMode::Auto => unreachable!(), // Already resolved above
        }
    }

    /// Render objects individually with vertex buffer caching
    fn render_individual<R: Renderable + VertexProvider>(
        &mut self,
        renderables: &[&R],
    ) -> Result<(), wgpu::SurfaceError> {
        let output = self.gpu.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Cache view-projection matrix calculation
        let view_projection_matrix = self.get_view_projection_matrix();

        // Render each object with cached vertex buffers
        for (i, renderable) in renderables.iter().enumerate() {
            let mut encoder = self.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some(&format!("Render Encoder {}", i)),
            });

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some(&format!("Render Pass {}", i)),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: if i == 0 {
                                wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 })
                            } else {
                                wgpu::LoadOp::Load
                            },
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

                // Calculate final transform
                let model_matrix = renderable.get_matrix();
                let final_matrix = view_projection_matrix * model_matrix;

                // Update uniform buffer
                self.gpu.queue.write_buffer(
                    &self.pipeline.uniform_buffer,
                    0,
                    bytemuck::cast_slice(final_matrix.as_ref()),
                );

                // Get cached vertex buffer (OPTIMIZATION!)
                let (vertex_buffer, vertex_count) = self.vertex_cache.get_or_create_buffer(*renderable, &self.gpu.device);

                render_pass.set_pipeline(&self.pipeline.pipeline);
                render_pass.set_bind_group(0, &self.pipeline.uniform_bind_group, &[]);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.draw(0..vertex_count, 0..1);
            }

            self.gpu.queue.submit(std::iter::once(encoder.finish()));
        }

        // Cleanup old cache entries periodically
        self.vertex_cache.cleanup_old_entries();

        output.present();
        Ok(())
    }

    /// Get performance statistics
    pub fn get_stats(&self) -> FlexibleRendererStats {
        let (cache_entries, cached_vertices) = self.vertex_cache.stats();
        FlexibleRendererStats {
            cache_entries,
            cached_vertices,
            current_mode: self.render_mode,
        }
    }
}

#[derive(Debug)]
pub struct FlexibleRendererStats {
    pub cache_entries: usize,
    pub cached_vertices: usize,
    pub current_mode: RenderMode,
}

impl std::fmt::Display for FlexibleRendererStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Renderer Stats: {} cached buffers, {} vertices, mode: {:?}", 
               self.cache_entries, self.cached_vertices, self.current_mode)
    }
}
