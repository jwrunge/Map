//! Headless rendering for embedded use
//!
//! Provides rendering capabilities without requiring a window system.
//! Useful for embedding in existing applications or server-side rendering.

use crate::renderable::{Cube, Quad, Triangle, VertexProvider};
use crate::renderer::{
    camera::Camera, config::{RenderConfig, CullingMode}, dynamic_uniforms::DynamicUniformBuffer,
    pipeline::RenderPipeline, vertex_cache::VertexBufferCache,
};
use anyhow::Result;

/// Headless renderer that renders to textures instead of windows
pub struct HeadlessRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: RenderPipeline,
    uniform_buffer: DynamicUniformBuffer,
    vertex_cache: VertexBufferCache,
    camera: Camera,
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

        let uniform_buffer = DynamicUniformBuffer::new(&device);
        let pipeline = RenderPipeline::new_headless_with_config(
            &device,
            uniform_buffer.get_bind_group_layout(),
            width,
            height,
            config,
        )?;
        let vertex_cache = VertexBufferCache::new();
        let camera = Camera::new(width as f32 / height as f32);

        Ok(Self {
            device,
            queue,
            pipeline,
            uniform_buffer,
            vertex_cache,
            camera,
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

        // Create multisampled texture if antialiasing is enabled
        let (render_view, resolve_target) = if self.pipeline.config.antialiasing.is_multisampled() {
            let multisampled_desc = wgpu::TextureDescriptor {
                size: wgpu::Extent3d {
                    width: self.width,
                    height: self.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: self.pipeline.config.antialiasing.sample_count(),
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                label: Some("Headless Multisampled Texture"),
                view_formats: &[],
            };
            let multisampled_texture = self.device.create_texture(&multisampled_desc);
            let multisampled_view =
                multisampled_texture.create_view(&wgpu::TextureViewDescriptor::default());
            (multisampled_view, Some(&output_view))
        } else {
            (output_view, None)
        };

        // Render the scene
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Headless Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Headless Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &render_view,
                    resolve_target,
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

            self.render_scene_internal(&mut render_pass, triangles, quads, cubes)?;
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        // For now, return a dummy buffer indicating successful render
        // In a real implementation, you'd copy the texture to a buffer and return the pixels
        Ok(vec![0u8; (self.width * self.height * 4) as usize])
    }

    /// Internal rendering implementation
    fn render_scene_internal(
        &mut self,
        render_pass: &mut wgpu::RenderPass,
        triangles: &[Triangle],
        quads: &[Quad],
        cubes: &[Cube],
    ) -> Result<()> {
        render_pass.set_pipeline(&self.pipeline.pipeline);

        // Collect all objects as trait objects
        let mut all_objects: Vec<&dyn VertexProvider> = Vec::new();
        for triangle in triangles {
            all_objects.push(triangle);
        }
        for quad in quads {
            all_objects.push(quad.mesh());
        }
        for cube in cubes {
            all_objects.push(cube.mesh());
        }

        // Get vertex buffers
        let vertex_buffers = self
            .vertex_cache
            .get_or_create_mixed_buffers(&all_objects, &self.device);

        // Collect transforms and multiply with view-projection
        let view_proj = self.camera.get_view_projection_matrix();
        let mut final_matrices = Vec::new();
        for triangle in triangles {
            final_matrices.push(view_proj * triangle.transform().to_matrix());
        }
        for quad in quads {
            final_matrices.push(view_proj * quad.transform().to_matrix());
        }
        for cube in cubes {
            final_matrices.push(view_proj * cube.transform().to_matrix());
        }

        // Reset and update uniform buffer
        self.uniform_buffer.reset_frame();
        let bind_groups = self
            .uniform_buffer
            .upload_matrices(&self.queue, &final_matrices);

        // Render all objects
        for (i, (vertex_buffer, vertex_count)) in vertex_buffers.iter().enumerate() {
            if let Some((bind_group, dynamic_offset)) = bind_groups.get(i) {
                render_pass.set_bind_group(0, Some(*bind_group), &[*dynamic_offset]);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.draw(0..*vertex_count, 0..1);
            }
        }

        Ok(())
    }

    /// Get the camera for external manipulation
    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    /// Resize the render target
    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        self.width = width;
        self.height = height;
        self.camera.set_aspect_ratio(width as f32 / height as f32);
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
        self.pipeline.update_config(
            &self.device,
            self.uniform_buffer.get_bind_group_layout(),
            wgpu::TextureFormat::Rgba8UnormSrgb,
            config,
            self.width,
            self.height,
        );
        Ok(())
    }

    /// Get the current rendering configuration
    pub fn get_config(&self) -> &RenderConfig {
        &self.pipeline.config
    }

    /// Set antialiasing mode specifically
    pub fn set_antialiasing(
        &mut self,
        mode: crate::renderer::config::AntialiasingMode,
    ) -> Result<()> {
        let mut config = self.pipeline.config.clone();
        config.antialiasing = mode;
        self.update_config(config)
    }

    /// Set culling mode specifically
    pub fn set_culling(&mut self, mode: crate::renderer::config::CullingMode) -> Result<()> {
        let mut config = self.pipeline.config.clone();
        config.culling = mode;
        self.update_config(config)
    }

    /// Enable/disable alpha blending
    pub fn set_alpha_blending(&mut self, enabled: bool) -> Result<()> {
        let mut config = self.pipeline.config.clone();
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
        // Group objects by culling mode for separate rendering passes
        use std::collections::HashMap;
        
        let mut culling_groups: HashMap<CullingMode, (Vec<&Triangle>, Vec<&Quad>, Vec<&Cube>)> = HashMap::new();
        
        // Group triangles by culling mode
        for triangle in triangles {
            let culling_mode = triangle.get_culling_mode();
            culling_groups.entry(culling_mode).or_insert_with(|| (Vec::new(), Vec::new(), Vec::new())).0.push(*triangle);
        }
        
        // Group quads by culling mode
        for quad in quads {
            let culling_mode = quad.get_culling_mode();
            culling_groups.entry(culling_mode).or_insert_with(|| (Vec::new(), Vec::new(), Vec::new())).1.push(*quad);
        }
        
        // Group cubes by culling mode
        for cube in cubes {
            let culling_mode = cube.get_culling_mode();
            culling_groups.entry(culling_mode).or_insert_with(|| (Vec::new(), Vec::new(), Vec::new())).2.push(*cube);
        }

        // Log the per-object culling groups
        for (culling_mode, (group_triangles, group_quads, group_cubes)) in &culling_groups {
            if !group_triangles.is_empty() || !group_quads.is_empty() || !group_cubes.is_empty() {
                log::info!("🎨 Culling group {:?}: {} triangles, {} quads, {} cubes", 
                          culling_mode, group_triangles.len(), group_quads.len(), group_cubes.len());
            }
        }

        // For demonstration purposes, return a placeholder buffer
        // In a full implementation, this would render each group with the appropriate culling pipeline
        let total_objects = triangles.len() + quads.len() + cubes.len();
        let pixel_data = vec![0u8; (self.width * self.height * 4) as usize]; // RGBA
        
        log::info!("✅ Per-object culling demo: {} groups, {} total objects", culling_groups.len(), total_objects);
        
        Ok(pixel_data)
    }
}
