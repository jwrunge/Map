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
pub mod vertex_cache;

#[cfg(feature = "windowing")]
use crate::renderable::{Renderable, VertexProvider, Triangle, Quad, Cube};
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
pub use vertex_cache::VertexBufferCache;

/// High-level renderer that coordinates GPU resources and scene rendering
#[cfg(feature = "windowing")]
pub struct Renderer {
    pub gpu: GpuContext,
    pub pipeline: RenderPipeline,
    pub camera: Camera,
    vertex_cache: VertexBufferCache,
    dynamic_uniforms: DynamicUniformBuffer,
}

#[cfg(feature = "windowing")]
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
                let (color_attachment_view, resolve_target) = if self.pipeline.config.antialiasing.is_multisampled() {
                    // Use multisampled framebuffer and resolve to surface
                    (self.pipeline.multisampled_framebuffer.as_ref().unwrap(), Some(&view))
                } else {
                    // Render directly to surface
                    (&view, None)
                };

                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: color_attachment_view,
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

    /// Render mixed object types (triangles, quads, cubes) in a single frame
    /// Objects are grouped by culling mode and rendered in separate passes to the same frame
    pub fn render_mixed_objects(
        &mut self,
        triangles: &[&Triangle],
        quads: &[&Quad], 
        cubes: &[&Cube],
    ) -> Result<(), wgpu::SurfaceError> {
        // If all collections are empty, nothing to render
        if triangles.is_empty() && quads.is_empty() && cubes.is_empty() {
            return Ok(());
        }

        // Group objects by culling mode for separate rendering passes
        use std::collections::HashMap;
        use crate::renderer::config::CullingMode;
        
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

        // Clear frame data ONCE for the entire frame
        self.dynamic_uniforms.reset_frame();

        // Extract bind group layout BEFORE any mutations
        let bind_group_layout = self.dynamic_uniforms.get_bind_group_layout().clone();

        // Collect ALL objects and matrices across all culling groups FIRST
        let mut all_objects_by_group: Vec<(CullingMode, Vec<&dyn VertexProvider>)> = Vec::new();
        let mut all_matrices: Vec<glam::Mat4> = Vec::new();

        for (culling_mode, (group_triangles, group_quads, group_cubes)) in &culling_groups {
            if group_triangles.is_empty() && group_quads.is_empty() && group_cubes.is_empty() {
                continue;
            }

            let mut group_objects: Vec<&dyn VertexProvider> = Vec::new();
            
            // Add matrices and objects in the same order
            for triangle in group_triangles {
                all_matrices.push(self.camera.get_view_projection_matrix() * triangle.get_matrix());
                group_objects.push(*triangle);
            }
            for quad in group_quads {
                all_matrices.push(self.camera.get_view_projection_matrix() * quad.get_matrix());
                group_objects.push(*quad);
            }
            for cube in group_cubes {
                all_matrices.push(self.camera.get_view_projection_matrix() * cube.get_matrix());
                group_objects.push(*cube);
            }

            all_objects_by_group.push((*culling_mode, group_objects));
        }

        // Create all vertex buffers and upload all uniforms FIRST
        let all_objects: Vec<&dyn VertexProvider> = all_objects_by_group.iter()
            .flat_map(|(_, objects)| objects.iter().cloned())
            .collect();
        
        let all_vertex_buffers = if !all_objects.is_empty() {
            self.vertex_cache.get_or_create_mixed_buffers(&all_objects, &self.gpu.device)
        } else {
            Vec::new()
        };

        let object_data = if !all_matrices.is_empty() {
            self.dynamic_uniforms.upload_matrices(&self.gpu.queue, &all_matrices)
        } else {
            Vec::new()
        };

        // Extract EVERYTHING needed for rendering before creating pipelines
        let gpu_device = &self.gpu.device;
        let gpu_format = self.gpu.config.format;
        let pipeline_config = self.pipeline.config.clone();
        let default_pipeline = &self.pipeline.pipeline;
        let multisampled_framebuffer = self.pipeline.multisampled_framebuffer.as_ref();

        // Create pipelines for all culling modes BEFORE rendering
        let mut pipelines: HashMap<CullingMode, wgpu::RenderPipeline> = HashMap::new();
        for (culling_mode, _) in &all_objects_by_group {
            if *culling_mode != pipeline_config.culling {
                let pipeline = RenderPipeline::create_culling_pipeline(
                    gpu_device,
                    &bind_group_layout,
                    gpu_format,
                    &pipeline_config,
                    *culling_mode,
                );
                pipelines.insert(*culling_mode, pipeline);
            }
        }

        // Now render each culling group using all pre-computed data
        let mut first_group = true;
        let mut object_index = 0;
        for (culling_mode, group_objects) in all_objects_by_group {
            let group_size = group_objects.len();
            if group_size > 0 {
                // Select the appropriate pipeline
                let pipeline = if let Some(temp_pipeline) = pipelines.get(&culling_mode) {
                    temp_pipeline
                } else {
                    default_pipeline
                };

                // Render this group using pre-computed data
                Self::render_culling_group_static(
                    &mut encoder,
                    &view,
                    pipeline,
                    multisampled_framebuffer,
                    &all_vertex_buffers[object_index..object_index + group_size],
                    &object_data[object_index..object_index + group_size],
                    first_group,
                    culling_mode,
                    &pipeline_config,
                )?;
                first_group = false;
                object_index += group_size;
            }
        }

        // Submit all rendering and present once
        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        // Periodic cache cleanup
        self.vertex_cache.cleanup_old_entries();

        Ok(())
    }

    /// Static rendering method that doesn't require borrowing self
    fn render_culling_group_static(
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        pipeline: &wgpu::RenderPipeline,
        multisampled_framebuffer: Option<&wgpu::TextureView>,
        vertex_buffers: &[(&wgpu::Buffer, u32)],
        uniform_data: &[(&wgpu::BindGroup, u32)],
        should_clear: bool,
        culling_mode: CullingMode,
        config: &RenderConfig,
    ) -> Result<(), wgpu::SurfaceError> {
        if vertex_buffers.is_empty() || uniform_data.is_empty() {
            return Ok(());
        }

        let (color_attachment_view, resolve_target) = if config.antialiasing.is_multisampled() {
            (multisampled_framebuffer.unwrap(), Some(view))
        } else {
            (view, None)
        };

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Culling Group Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_attachment_view,
                resolve_target,
                ops: wgpu::Operations {
                    load: if should_clear {
                        wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        })
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

        render_pass.set_pipeline(pipeline);

        // Render each object
        for (index, vertex_buffer) in vertex_buffers.iter().enumerate() {
            if index < uniform_data.len() {
                let (bind_group, dynamic_offset) = &uniform_data[index];

                render_pass.set_vertex_buffer(0, vertex_buffer.0.slice(..));
                render_pass.set_bind_group(0, *bind_group, &[*dynamic_offset]);
                render_pass.draw(0..vertex_buffer.1, 0..1);
            }
        }

        log::info!("🎨 Rendered {} objects with culling mode {:?}", vertex_buffers.len(), culling_mode);

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

    /// Update the rendering configuration (antialiasing, culling, etc.)
    pub fn update_config(&mut self, config: RenderConfig) {
        self.pipeline.update_config(
            &self.gpu.device,
            self.dynamic_uniforms.get_bind_group_layout(),
            self.gpu.config.format,
            config,
            self.gpu.config.width,
            self.gpu.config.height,
        );
    }

    /// Get the current rendering configuration
    pub fn get_config(&self) -> &RenderConfig {
        &self.pipeline.config
    }

    /// Set antialiasing mode specifically
    pub fn set_antialiasing(&mut self, mode: AntialiasingMode) {
        let mut config = self.pipeline.config.clone();
        config.antialiasing = mode;
        self.update_config(config);
    }

    /// Set culling mode specifically
    pub fn set_culling(&mut self, mode: CullingMode) {
        let mut config = self.pipeline.config.clone();
        config.culling = mode;
        self.update_config(config);
    }

    /// Enable/disable alpha blending
    pub fn set_alpha_blending(&mut self, enabled: bool) {
        let mut config = self.pipeline.config.clone();
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
