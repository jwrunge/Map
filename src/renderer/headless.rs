//! Headless rendering for embedded use
//!
//! Provides rendering capabilities without requiring a window system.
//! Useful for embedding in existing applications or server-side rendering.

use crate::renderable::{Cube, Quad, Triangle};
use crate::renderer::{
    config::RenderConfig, 
    render_core::RenderCore,
};
use anyhow::Result;

/// Headless renderer that renders to textures instead of windows
pub struct HeadlessRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_core: RenderCore,
    width: u32,
    height: u32,
}

impl HeadlessRenderer {
    /// Create a new headless renderer with the specified dimensions
    pub async fn new(width: u32, height: u32) -> Result<Self> {
        Self::new_with_config(width, height, RenderConfig::default()).await
    }

    /// Create a new headless renderer with custom configuration
    pub async fn new_with_config(width: u32, height: u32, config: RenderConfig) -> Result<Self> {
        // Create a headless wgpu instance
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None, // No surface needed for headless
                force_fallback_adapter: false,
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                ..Default::default()
            })
            .await?;

        let render_core = RenderCore::new_headless(&device, width, height, config)?;

        Ok(Self {
            device,
            queue,
            render_core,
            width,
            height,
        })
    }

    /// Render scene to a texture (simplified version for demo)
    /// Returns success/failure rather than pixel data for now
    pub fn render_to_buffer(
        &mut self,
        triangles: &[Triangle],
        quads: &[Quad],
        cubes: &[Cube],
    ) -> Result<Vec<u8>> {
        // Create output texture
        let texture_desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1, // Output texture is always single-sampled
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("Headless Render Texture"),
            view_formats: &[],
        };
        let output_texture = self.device.create_texture(&texture_desc);
        let output_view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Render the scene
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Headless Render Encoder"),
            });

        // Convert to reference slices for RenderCore
        let triangle_refs: Vec<&Triangle> = triangles.iter().collect();
        let quad_refs: Vec<&Quad> = quads.iter().collect();
        let cube_refs: Vec<&Cube> = cubes.iter().collect();

        // Use the shared render core
        self.render_core.render_mixed_objects_core(
            &self.device,
            &self.queue,
            &mut encoder,
            &output_view,
            None, // No multisampling in basic implementation
            &triangle_refs,
            &quad_refs,
            &cube_refs,
            true, // should_clear
        )?;

        self.queue.submit(std::iter::once(encoder.finish()));

        // For now, return a dummy buffer indicating successful render
        // In a real implementation, you'd copy the texture to a buffer and return the pixels
        Ok(vec![0u8; (self.width * self.height * 4) as usize])
    }

    /// Get the camera for external manipulation
    pub fn camera_mut(&mut self) -> &mut crate::renderer::camera::Camera {
        self.render_core.camera_mut()
    }

    /// Resize the render target
    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        self.width = width;
        self.height = height;
        self.render_core.resize(width, height);
        Ok(())
    }

    /// Get current dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get access to the underlying device for capability queries
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Update the rendering configuration (antialiasing, culling, etc.)
    pub fn update_config(&mut self, config: RenderConfig) -> Result<()> {
        self.render_core.update_config(
            &self.device,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            config,
            self.width,
            self.height,
        );
        Ok(())
    }

    /// Get the current rendering configuration
    pub fn get_config(&self) -> &RenderConfig {
        self.render_core.get_config()
    }

    /// Set antialiasing mode specifically
    pub fn set_antialiasing(
        &mut self,
        mode: crate::renderer::config::AntialiasingMode,
    ) -> Result<()> {
        let mut config = self.render_core.get_config().clone();
        config.antialiasing = mode;
        self.update_config(config)
    }

    /// Set culling mode specifically
    pub fn set_culling(&mut self, mode: crate::renderer::config::CullingMode) -> Result<()> {
        let mut config = self.render_core.get_config().clone();
        config.culling = mode;
        self.update_config(config)
    }

    /// Enable/disable alpha blending
    pub fn set_alpha_blending(&mut self, enabled: bool) -> Result<()> {
        let mut config = self.render_core.get_config().clone();
        config.alpha_blending = enabled;
        self.update_config(config)
    }

    /// Switch to 2D optimized settings (no backface culling, alpha blending)
    pub fn set_2d_mode(&mut self) -> Result<()> {
        self.update_config(RenderConfig::for_2d())
    }

    /// Switch to 3D optimized settings (backface culling, no alpha blending)
    pub fn set_3d_mode(&mut self) -> Result<()> {
        self.update_config(RenderConfig::for_3d())
    }

    /// Switch to performance mode (no antialiasing)
    pub fn set_performance_mode(&mut self) -> Result<()> {
        self.update_config(RenderConfig::performance())
    }

    /// Render mixed object types with per-object culling support
    /// Objects are grouped by culling mode and rendered in separate passes
    pub fn render_mixed_objects_to_buffer(
        &mut self,
        triangles: &[&Triangle],
        quads: &[&Quad],
        cubes: &[&Cube],
    ) -> Result<Vec<u8>> {
        // Create output texture
        let texture_desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("Headless Mixed Objects Texture"),
            view_formats: &[],
        };
        let output_texture = self.device.create_texture(&texture_desc);
        let output_view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Render the scene
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Headless Mixed Objects Encoder"),
            });

        // Use the shared render core for actual per-object culling rendering
        self.render_core.render_mixed_objects_core(
            &self.device,
            &self.queue,
            &mut encoder,
            &output_view,
            None, // No multisampling in basic implementation
            triangles,
            quads,
            cubes,
            true, // should_clear
        )?;

        self.queue.submit(std::iter::once(encoder.finish()));

        // Log completion
        let total_objects = triangles.len() + quads.len() + cubes.len();
        log::info!("✅ Headless per-object culling render: {} total objects", total_objects);
        
        // Return placeholder pixel data
        Ok(vec![0u8; (self.width * self.height * 4) as usize])
    }
}
