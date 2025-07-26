//! Core rendering logic shared between windowed and headless renderers
//!
//! This module contains the unified rendering implementation that eliminates
//! code duplication between windowed and headless rendering modes.

use std::collections::HashMap;
use crate::renderable::{VertexProvider, Triangle, Quad, Cube, Circle, Cylinder, Cone, Sphere, Renderable};
use crate::renderer::{
    config::{RenderConfig, CullingMode}, 
    dynamic_uniforms::DynamicUniformBuffer,
    pipeline::RenderPipeline, 
    vertex_cache::VertexBufferCache,
    camera::Camera,
};

/// Shared rendering logic and resources
pub struct RenderCore {
    pub pipeline: RenderPipeline,
    pub uniform_buffer: DynamicUniformBuffer,
    pub vertex_cache: VertexBufferCache,
    pub camera: Camera,
}

impl RenderCore {
    /// Create a new render core (windowed)
    #[cfg(feature = "windowing")]
    pub fn new_windowed(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        config: RenderConfig,
    ) -> Self {
        let uniform_buffer = DynamicUniformBuffer::new(device);
        
        // Temporarily disable MSAA for windowed rendering to fix compatibility issue
        let mut windowed_config = config;
        windowed_config.antialiasing = crate::renderer::config::AntialiasingMode::None;
        
        let pipeline = RenderPipeline::new_with_config_core(
            device,
            uniform_buffer.get_bind_group_layout(),
            format,
            windowed_config,
            width,
            height,
        );
        let camera = Camera::new(width as f32 / height as f32);

        Self {
            pipeline,
            uniform_buffer,
            vertex_cache: VertexBufferCache::new(),
            camera,
        }
    }

    /// Create a new render core (headless)
    #[cfg(feature = "headless")]
    pub fn new_headless(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        config: RenderConfig,
    ) -> anyhow::Result<Self> {
        let uniform_buffer = DynamicUniformBuffer::new(device);
        let pipeline = RenderPipeline::new_headless_with_config(
            device,
            uniform_buffer.get_bind_group_layout(),
            width,
            height,
            config,
        )?;
        let camera = Camera::new(width as f32 / height as f32);

        Ok(Self {
            pipeline,
            uniform_buffer,
            vertex_cache: VertexBufferCache::new(),
            camera,
        })
    }

    /// Unified mixed object rendering implementation
    /// This is the core logic used by both windowed and headless renderers
    pub fn render_mixed_objects_core(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        multisampled_view: Option<&wgpu::TextureView>,
        triangles: &[&Triangle],
        quads: &[&Quad],
        cubes: &[&Cube],
        circles: &[&Circle],
        cylinders: &[&Cylinder],
        cones: &[&Cone],
        spheres: &[&Sphere],
        should_clear: bool,
    ) -> Result<(), wgpu::SurfaceError> {
        // Early exit if nothing to render
        if triangles.is_empty() && quads.is_empty() && cubes.is_empty() && 
           circles.is_empty() && cylinders.is_empty() && cones.is_empty() && 
           spheres.is_empty() {
            return Ok(());
        }

        // Clear frame data ONCE for the entire frame
        self.uniform_buffer.reset_frame();

        // Extract bind group layout BEFORE any mutations
        let bind_group_layout = self.uniform_buffer.get_bind_group_layout().clone();

        // Group objects by culling mode for separate rendering passes
        let culling_groups = self.organize_by_culling_mode(triangles, quads, cubes, circles, cylinders, cones, spheres);

        // Collect ALL objects and matrices across all culling groups
        let (all_objects_by_group, all_matrices) = Self::collect_objects_and_matrices_static(&self.camera, &culling_groups);

        // Create all vertex buffers and upload all uniforms
        let all_objects: Vec<&dyn VertexProvider> = all_objects_by_group.iter()
            .flat_map(|(_, objects)| objects.iter().cloned())
            .collect();
        
        let all_vertex_buffers = if !all_objects.is_empty() {
            self.vertex_cache.get_or_create_mixed_buffers(&all_objects, device)
        } else {
            Vec::new()
        };

        let object_data = if !all_matrices.is_empty() {
            self.uniform_buffer.upload_matrices(queue, &all_matrices)
        } else {
            Vec::new()
        };

        // Extract everything needed for rendering before creating pipelines
        let pipeline_config = self.pipeline.config.clone();
        let default_pipeline = &self.pipeline.pipeline;

        // Create pipelines for all culling modes BEFORE rendering (skip MSAA for now)
        let mut pipelines: HashMap<CullingMode, wgpu::RenderPipeline> = HashMap::new();
        for (culling_mode, _) in &all_objects_by_group {
            if *culling_mode != pipeline_config.culling {
                // Create a temporary config without MSAA for compatibility
                let mut temp_config = pipeline_config.clone();
                temp_config.antialiasing = crate::renderer::config::AntialiasingMode::None;
                
                let pipeline = RenderPipeline::create_culling_pipeline(
                    device,
                    &bind_group_layout,
                    self.pipeline.get_format(),
                    &temp_config,
                    *culling_mode,
                );
                pipelines.insert(*culling_mode, pipeline);
            }
        }

        // Now render each culling group using all pre-computed data
        let mut first_group = should_clear;
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
                    encoder,
                    target_view,
                    depth_view,
                    pipeline,
                    multisampled_view,
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

        // Periodic cache cleanup
        self.vertex_cache.cleanup_old_entries();

        Ok(())
    }

    /// Organize objects by culling mode for batched rendering
    fn organize_by_culling_mode<'a>(
        &self,
        triangles: &'a [&Triangle],
        quads: &'a [&Quad],
        cubes: &'a [&Cube],
        circles: &'a [&Circle],
        cylinders: &'a [&Cylinder],
        cones: &'a [&Cone],
        spheres: &'a [&Sphere],
    ) -> HashMap<CullingMode, (Vec<&'a Triangle>, Vec<&'a Quad>, Vec<&'a Cube>, Vec<&'a Circle>, Vec<&'a Cylinder>, Vec<&'a Cone>, Vec<&'a Sphere>)> {
        let mut culling_groups: HashMap<CullingMode, (Vec<&Triangle>, Vec<&Quad>, Vec<&Cube>, Vec<&Circle>, Vec<&Cylinder>, Vec<&Cone>, Vec<&Sphere>)> = HashMap::new();
        
        // Group triangles by culling mode
        for triangle in triangles {
            let culling_mode = triangle.get_culling_mode();
            culling_groups.entry(culling_mode).or_insert_with(|| (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new())).0.push(*triangle);
        }
        
        // Group quads by culling mode
        for quad in quads {
            let culling_mode = quad.get_culling_mode();
            culling_groups.entry(culling_mode).or_insert_with(|| (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new())).1.push(*quad);
        }
        
        // Group cubes by culling mode
        for cube in cubes {
            let culling_mode = cube.get_culling_mode();
            culling_groups.entry(culling_mode).or_insert_with(|| (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new())).2.push(*cube);
        }

        // Group circles by culling mode
        for circle in circles {
            let culling_mode = circle.get_culling_mode();
            culling_groups.entry(culling_mode).or_insert_with(|| (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new())).3.push(*circle);
        }

        // Group cylinders by culling mode
        for cylinder in cylinders {
            let culling_mode = cylinder.get_culling_mode();
            culling_groups.entry(culling_mode).or_insert_with(|| (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new())).4.push(*cylinder);
        }

        // Group cones by culling mode
        for cone in cones {
            let culling_mode = cone.get_culling_mode();
            culling_groups.entry(culling_mode).or_insert_with(|| (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new())).5.push(*cone);
        }

        // Group spheres by culling mode
        for sphere in spheres {
            let culling_mode = sphere.get_culling_mode();
            culling_groups.entry(culling_mode).or_insert_with(|| (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new())).6.push(*sphere);
        }

        culling_groups
    }

    /// Collect objects and calculate transformation matrices (static version to avoid borrowing issues)
    fn collect_objects_and_matrices_static<'a>(
        camera: &Camera,
        culling_groups: &'a HashMap<CullingMode, (Vec<&'a Triangle>, Vec<&'a Quad>, Vec<&'a Cube>, Vec<&'a Circle>, Vec<&'a Cylinder>, Vec<&'a Cone>, Vec<&'a Sphere>)>,
    ) -> (Vec<(CullingMode, Vec<&'a dyn VertexProvider>)>, Vec<glam::Mat4>) {
        let mut all_objects_by_group: Vec<(CullingMode, Vec<&dyn VertexProvider>)> = Vec::new();
        let mut all_matrices: Vec<glam::Mat4> = Vec::new();

        for (culling_mode, (group_triangles, group_quads, group_cubes, group_circles, group_cylinders, group_cones, group_spheres)) in culling_groups {
            if group_triangles.is_empty() && group_quads.is_empty() && group_cubes.is_empty() && 
               group_circles.is_empty() && group_cylinders.is_empty() && group_cones.is_empty() && 
               group_spheres.is_empty() {
                continue;
            }

            let mut group_objects: Vec<&dyn VertexProvider> = Vec::new();
            
            // Add matrices and objects in the same order
            for mut triangle in group_triangles {
                all_matrices.push(camera.get_view_projection_matrix() * triangle.get_matrix_cached());
                group_objects.push(*triangle);
            }
            for mut quad in group_quads {
                all_matrices.push(camera.get_view_projection_matrix() * quad.get_matrix_cached());
                group_objects.push(*quad);
            }
            for mut cube in group_cubes {
                all_matrices.push(camera.get_view_projection_matrix() * cube.get_matrix_cached());
                group_objects.push(*cube);
            }
            for mut circle in group_circles {
                all_matrices.push(camera.get_view_projection_matrix() * circle.get_matrix_cached());
                group_objects.push(*circle);
            }
            for mut cylinder in group_cylinders {
                all_matrices.push(camera.get_view_projection_matrix() * cylinder.get_matrix_cached());
                group_objects.push(*cylinder);
            }
            for mut cone in group_cones {
                all_matrices.push(camera.get_view_projection_matrix() * cone.get_matrix_cached());
                group_objects.push(*cone);
            }
            for mut sphere in group_spheres {
                all_matrices.push(camera.get_view_projection_matrix() * sphere.get_matrix_cached());
                group_objects.push(*sphere);
            }

            all_objects_by_group.push((*culling_mode, group_objects));
        }

        (all_objects_by_group, all_matrices)
    }

    /// Static rendering method that doesn't require borrowing self
    fn render_culling_group_static(
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        pipeline: &wgpu::RenderPipeline,
        _multisampled_framebuffer: Option<&wgpu::TextureView>,
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
            // In a real implementation, we would need access to the multisampled framebuffer
            // For now, fall back to direct rendering to avoid borrowing conflicts
            (view, None)
        } else {
            (view, None)
        };

        {
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: if should_clear {
                            wgpu::LoadOp::Clear(1.0)
                        } else {
                            wgpu::LoadOp::Load
                        },
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
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

            // Render pass is automatically dropped here
        }

        log::debug!("🎨 Rendered {} objects with culling mode {:?}", vertex_buffers.len(), culling_mode);

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

    /// Update the rendering configuration
    pub fn update_config(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        config: RenderConfig,
        width: u32,
        height: u32,
    ) {
        self.pipeline.update_config(
            device,
            self.uniform_buffer.get_bind_group_layout(),
            format,
            config,
            width,
            height,
        );
    }

    /// Get the current rendering configuration
    pub fn get_config(&self) -> &RenderConfig {
        &self.pipeline.config
    }

    /// Get camera for external manipulation
    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    /// Get mutable camera for external manipulation
    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    /// Resize the render target
    pub fn resize(&mut self, width: u32, height: u32) {
        self.camera.set_aspect_ratio(width as f32 / height as f32);
    }
}
