//! Graphics rendering system
//!
//! This module provides the core rendering infrastructure, separating
//! GPU resource management from scene/entity management.

pub mod camera;
pub mod gpu_context;
pub mod pipeline;

use crate::renderable::{Renderable, VertexProvider};
use wgpu::util::DeviceExt;
use winit::window::Window;

pub use camera::Camera;
pub use gpu_context::GpuContext;
pub use pipeline::RenderPipeline;
/// High-level renderer that coordinates GPU resources and scene rendering
pub struct Renderer {
    pub gpu: GpuContext,
    pub pipeline: RenderPipeline,
    pub camera: Camera,
}

impl Renderer {
    pub async fn new(window: std::sync::Arc<Window>) -> Self {
        let gpu = GpuContext::new(window).await;
        let pipeline = RenderPipeline::new(&gpu);
        let camera = Camera::new(gpu.size.width as f32 / gpu.size.height as f32);

        Self {
            gpu,
            pipeline,
            camera,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.gpu.resize(new_size);
        self.camera
            .set_aspect_ratio(new_size.width as f32 / new_size.height as f32);
    }

    pub fn render<R: Renderable + VertexProvider>(
        &mut self,
        renderable: &R,
    ) -> Result<(), wgpu::SurfaceError> {
        self.pipeline.render(&self.gpu, &self.camera, renderable)
    }

    /// Render multiple objects efficiently by reusing surface texture
    pub fn render_batch<R: Renderable + VertexProvider>(
        &mut self,
        renderables: &[&R],
    ) -> Result<(), wgpu::SurfaceError> {
        log::debug!("render_batch called with {} objects", renderables.len());

        if renderables.is_empty() {
            log::warn!("render_batch called with no objects!");
            return Ok(());
        }

        let output = self.gpu.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Render each object in its own render pass but reuse the surface texture
        for (i, renderable) in renderables.iter().enumerate() {
            log::debug!("Rendering object {} of {}", i + 1, renderables.len());

            let mut encoder =
                self.gpu
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
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
                                // Clear only for the first triangle
                                wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.1,
                                    g: 0.2,
                                    b: 0.3,
                                    a: 1.0,
                                })
                            } else {
                                // Load existing content for subsequent triangles
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

                // Combine camera projection with object transform
                let model_matrix = renderable.get_transform().to_matrix();
                let view_projection_matrix = self.camera.get_view_projection_matrix();
                let final_matrix = view_projection_matrix * model_matrix;

                // Update uniform buffer with this triangle's transform
                self.gpu.queue.write_buffer(
                    &self.pipeline.uniform_buffer,
                    0,
                    bytemuck::cast_slice(final_matrix.as_ref()),
                );

                // Create vertex buffer
                let vertex_buffer =
                    self.gpu
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Vertex Buffer"),
                            contents: renderable.buffer_contents(),
                            usage: wgpu::BufferUsages::VERTEX,
                        });

                render_pass.set_pipeline(&self.pipeline.pipeline);
                render_pass.set_bind_group(0, &self.pipeline.uniform_bind_group, &[]);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.draw(0..renderable.vertex_count() as u32, 0..1);
            }

            // Submit this triangle's commands
            self.gpu.queue.submit(std::iter::once(encoder.finish()));
        }

        // Present the final result
        output.present();
        Ok(())
    }
}
