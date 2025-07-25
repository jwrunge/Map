//! Graphics rendering system
//!
//! This module provides the core rendering infrastructure, separating
//! GPU resource management from scene/entity management.

pub mod camera;
pub mod config;
pub mod dynamic_uniforms;
#[cfg(feature = "windowing")]
pub mod gpu_context;
#[cfg(feature = "headless")]
pub mod headless;
pub mod pipeline;
pub mod render_core;
pub mod vertex_cache;

#[cfg(feature = "windowing")]
use crate::renderable::{Triangle, Quad, Cube};
#[cfg(feature = "windowing")]
use winit::window::Window;

pub use camera::Camera;
pub use config::{AntialiasingMode, CullingMode, RenderConfig};
#[cfg(feature = "headless")]
pub use headless::HeadlessRenderer;
pub use dynamic_uniforms::DynamicUniformBuffer;
#[cfg(feature = "windowing")]
pub use gpu_context::GpuContext;
pub use pipeline::RenderPipeline;
pub use render_core::RenderCore;
pub use vertex_cache::VertexBufferCache;

/// High-level renderer that coordinates GPU resources and scene rendering
#[cfg(feature = "windowing")]
pub struct Renderer {
    pub gpu: GpuContext,
    render_core: RenderCore,
}

#[cfg(feature = "windowing")]
impl Renderer {
    pub async fn new(window: std::sync::Arc<Window>) -> Self {
        let gpu = GpuContext::new(window).await;
        let render_core = RenderCore::new_windowed(
            &gpu.device,
            gpu.config.format,
            gpu.config.width,
            gpu.config.height,
            RenderConfig::default(),
        );

        Self {
            gpu,
            render_core,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.gpu.resize(new_size);
        self.render_core.resize(new_size.width, new_size.height);
    }

    /// Get the camera for external manipulation
    pub fn camera(&self) -> &Camera {
        self.render_core.camera()
    }

    /// Get mutable camera for external manipulation
    pub fn camera_mut(&mut self) -> &mut Camera {
        self.render_core.camera_mut()
    }

    /// Render mixed object types (triangles, quads, cubes) in a single frame
    /// Objects are grouped by culling mode and rendered in separate passes to the same frame
    pub fn render_mixed_objects(
        &mut self,
        triangles: &[&Triangle],
        quads: &[&Quad], 
        cubes: &[&Cube],
    ) -> Result<(), wgpu::SurfaceError> {
        // Create single frame output and encoder for all groups
        let output = self.gpu.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Mixed Objects Frame Encoder"),
            });

        // Use the shared render core for the actual rendering
        // Pass None for multisampled_framebuffer to avoid borrowing conflicts - 
        // the core will handle it internally
        self.render_core.render_mixed_objects_core(
            &self.gpu.device,
            &self.gpu.queue,
            &mut encoder,
            &view,
            None, // Will be handled inside the core based on config
            triangles,
            quads,
            cubes,
            true, // should_clear
        )?;

        // Submit all rendering and present once
        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Get renderer performance statistics
    pub fn get_cache_stats(&self) -> (usize, usize) {
        self.render_core.get_cache_stats()
    }

    /// Clear vertex buffer cache (useful for memory management)
    pub fn clear_cache(&mut self) {
        self.render_core.clear_cache();
    }

    /// Update the rendering configuration (antialiasing, culling, etc.)
    pub fn update_config(&mut self, config: RenderConfig) {
        self.render_core.update_config(
            &self.gpu.device,
            self.gpu.config.format,
            config,
            self.gpu.config.width,
            self.gpu.config.height,
        );
    }

    /// Get the current rendering configuration
    pub fn get_config(&self) -> &RenderConfig {
        self.render_core.get_config()
    }

    /// Set antialiasing mode specifically
    pub fn set_antialiasing(&mut self, mode: AntialiasingMode) {
        let mut config = self.render_core.get_config().clone();
        config.antialiasing = mode;
        self.update_config(config);
    }

    /// Set culling mode specifically
    pub fn set_culling(&mut self, mode: CullingMode) {
        let mut config = self.render_core.get_config().clone();
        config.culling = mode;
        self.update_config(config);
    }

    /// Enable/disable alpha blending
    pub fn set_alpha_blending(&mut self, enabled: bool) {
        let mut config = self.render_core.get_config().clone();
        config.alpha_blending = enabled;
        self.update_config(config);
    }

    /// Switch to 2D optimized settings (no backface culling, alpha blending)
    pub fn set_2d_mode(&mut self) {
        self.update_config(RenderConfig::for_2d());
    }

    /// Switch to 3D optimized settings (backface culling, no alpha blending)
    pub fn set_3d_mode(&mut self) {
        self.update_config(RenderConfig::for_3d());
    }

    /// Switch to performance mode (no antialiasing)
    pub fn set_performance_mode(&mut self) {
        self.update_config(RenderConfig::performance());
    }
}
