//! Graphics rendering system
//!
//! This module provides the core rendering infrastructure, separating
//! GPU resource management from scene/entity management.

pub mod camera;
pub mod gpu_context;
pub mod pipeline;

use crate::renderable::{Renderable, VertexProvider};
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
}
