//! Graphics rendering system
//!
//! This module provides the core rendering infrastructure, separating
//! GPU resource management from scene/entity management.

pub mod camera;
pub mod dynamic_uniforms;
pub mod gpu_context;
pub mod pipeline;
pub mod vertex_cache;

use crate::renderable::{Renderable, VertexProvider};
use winit::window::Window;

pub use camera::Camera;
pub use dynamic_uniforms::DynamicUniformBuffer;
pub use gpu_context::GpuContext;
pub use pipeline::RenderPipeline;
pub use vertex_cache::VertexBufferCache;
/// High-level renderer that coordinates GPU resources and scene rendering
pub struct Renderer {
    pub gpu: GpuContext,
    pub pipeline: RenderPipeline,
    pub camera: Camera,
    vertex_cache: VertexBufferCache,
    dynamic_uniforms: DynamicUniformBuffer,
}

impl Renderer {
    pub async fn new(window: std::sync::Arc<Window>) -> Self {
        let gpu = GpuContext::new(window).await;
        let dynamic_uniforms = DynamicUniformBuffer::new(&gpu.device);
        let pipeline = RenderPipeline::new(&gpu, dynamic_uniforms.get_bind_group_layout());
        let camera = Camera::new(800.0 / 600.0); // Default aspect ratio

        Self {
            gpu,
            pipeline,
            camera,
            vertex_cache: VertexBufferCache::new(),
            dynamic_uniforms,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.gpu.resize(new_size);
        self.camera
            .set_aspect_ratio(new_size.width as f32 / new_size.height as f32);
    }

    /// High-performance batch rendering with dynamic uniform buffers
    /// Renders all objects in a single pass using dynamic offsets
    pub fn render_batch_dynamic<R: Renderable + VertexProvider>(
        &mut self,
        renderables: &[R],
    ) -> Result<(), wgpu::SurfaceError> {
        // Clear frame data
        self.dynamic_uniforms.reset_frame();

        let output = self.gpu.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Check if any objects are dirty
        let mut has_updates = false;
        for renderable in renderables.iter() {
            if renderable.is_dirty() {
                has_updates = true;
                break;
            }
        }

        if has_updates {
            // Process each object individually to avoid borrowing issues

            // Step 1: Upload all matrices using batch method
            let transform_matrices: Vec<_> = renderables
                .iter()
                .map(|renderable| {
                    self.camera.get_view_projection_matrix() * renderable.get_matrix()
                })
                .collect();

            let object_data = self
                .dynamic_uniforms
                .upload_matrices(&self.gpu.queue, &transform_matrices);
            let successful_uploads = object_data.len();

            // Step 2: Create vertex buffers for all objects using batch method
            let renderables_to_process = &renderables[..successful_uploads.min(renderables.len())];
            let vertex_buffers = self
                .vertex_cache
                .get_or_create_multiple_buffers(renderables_to_process, &self.gpu.device);

            // Optional: Log render statistics (can be removed for production)
            // let total_rendered = vertex_buffers.len();
            // println!("🚀 Rendering with dynamic uniforms: {} objects prepared, {} rendered", successful_uploads, total_rendered);

            // Single render pass for all objects
            if !vertex_buffers.is_empty() && !object_data.is_empty() {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
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

                // Render each object with its dynamic offset
                for (index, vertex_buffer) in vertex_buffers.iter().enumerate() {
                    if index < object_data.len() {
                        let (bind_group, dynamic_offset) = &object_data[index];

                        // Set vertex buffer
                        render_pass.set_vertex_buffer(0, vertex_buffer.0.slice(..));

                        // Set uniform bind group with dynamic offset
                        render_pass.set_bind_group(0, *bind_group, &[*dynamic_offset]);

                        // Draw
                        let vertex_count = vertex_buffer.1;
                        render_pass.draw(0..vertex_count, 0..1);
                    }
                }
            }

            // Note: Cannot mark objects as clean since renderables is immutable
            // The caller should handle dirty flag management separately
        }

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        // Periodic cache cleanup
        self.vertex_cache.cleanup_old_entries();

        Ok(())
    }

    /// Get renderer performance statistics
    pub fn get_cache_stats(&self) -> (usize, usize) {
        self.vertex_cache.stats()
    }

    /// Clear vertex buffer cache (useful for memory management)
    pub fn clear_cache(&mut self) {
        self.vertex_cache = VertexBufferCache::new();
    }
}
