//! Render pipeline and shader management
//!
//! Handles the graphics pipeline setup and rendering operations

use crate::renderable::Vertex;
use crate::renderer::config::RenderConfig;

#[cfg(feature = "windowing")]
use crate::renderer::GpuContext;

/// Manages the rendering pipeline and draw operations
pub struct RenderPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub config: RenderConfig,
    pub format: wgpu::TextureFormat,
    #[allow(dead_code)]
    pub multisampled_framebuffer: Option<wgpu::TextureView>,
}

impl RenderPipeline {
    #[cfg(feature = "windowing")]
    pub fn new(gpu: &GpuContext, uniform_bind_group_layout: &wgpu::BindGroupLayout) -> Self {
        Self::new_with_config(gpu, uniform_bind_group_layout, RenderConfig::default())
    }

    #[cfg(feature = "windowing")]
    pub fn new_with_config(
        gpu: &GpuContext,
        uniform_bind_group_layout: &wgpu::BindGroupLayout,
        config: RenderConfig,
    ) -> Self {
        let (pipeline, multisampled_framebuffer) = Self::create_pipeline_with_config(
            &gpu.device,
            uniform_bind_group_layout,
            gpu.config.format,
            &config,
            gpu.config.width,
            gpu.config.height,
        );
        Self {
            pipeline,
            config,
            format: gpu.config.format,
            multisampled_framebuffer,
        }
    }

    #[cfg(feature = "headless")]
    pub fn new_headless(
        device: &wgpu::Device,
        uniform_bind_group_layout: &wgpu::BindGroupLayout,
        width: u32,
        height: u32,
    ) -> anyhow::Result<Self> {
        Self::new_headless_with_config(
            device,
            uniform_bind_group_layout,
            width,
            height,
            RenderConfig::default(),
        )
    }

    #[cfg(feature = "headless")]
    pub fn new_headless_with_config(
        device: &wgpu::Device,
        uniform_bind_group_layout: &wgpu::BindGroupLayout,
        width: u32,
        height: u32,
        config: RenderConfig,
    ) -> anyhow::Result<Self> {
        let (pipeline, multisampled_framebuffer) = Self::create_pipeline_with_config(
            device,
            uniform_bind_group_layout,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            &config,
            width,
            height,
        );
        Ok(Self {
            pipeline,
            config,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            multisampled_framebuffer,
        })
    }

    fn create_pipeline_with_config(
        device: &wgpu::Device,
        uniform_bind_group_layout: &wgpu::BindGroupLayout,
        format: wgpu::TextureFormat,
        config: &RenderConfig,
        width: u32,
        height: u32,
    ) -> (wgpu::RenderPipeline, Option<wgpu::TextureView>) {
        // Load shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Basic Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/basic.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[uniform_bind_group_layout],
                push_constant_ranges: &[],
            });

        // Create multisampled framebuffer if needed
        let multisampled_framebuffer = if config.antialiasing.is_multisampled() {
            let multisampled_texture_desc = wgpu::TextureDescriptor {
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: config.antialiasing.sample_count(),
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                label: Some("Multisampled framebuffer"),
                view_formats: &[],
            };
            let multisampled_texture = device.create_texture(&multisampled_texture_desc);
            Some(multisampled_texture.create_view(&wgpu::TextureViewDescriptor::default()))
        } else {
            None
        };

        // Configure blend state based on alpha blending setting
        let blend_state = if config.alpha_blending {
            Some(wgpu::BlendState::ALPHA_BLENDING)
        } else {
            Some(wgpu::BlendState::REPLACE)
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: blend_state,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: config.culling.to_wgpu(),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: config.antialiasing.sample_count(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        (pipeline, multisampled_framebuffer)
    }

    /// Update the render configuration and recreate the pipeline
    pub fn update_config(
        &mut self,
        device: &wgpu::Device,
        uniform_bind_group_layout: &wgpu::BindGroupLayout,
        format: wgpu::TextureFormat,
        config: RenderConfig,
        width: u32,
        height: u32,
    ) {
        let (pipeline, multisampled_framebuffer) = Self::create_pipeline_with_config(
            device,
            uniform_bind_group_layout,
            format,
            &config,
            width,
            height,
        );
        self.pipeline = pipeline;
        self.config = config;
        self.format = format;
        self.multisampled_framebuffer = multisampled_framebuffer;
    }

    /// Create a temporary pipeline with a specific culling mode
    /// Used for per-object culling support
    pub fn create_culling_pipeline(
        device: &wgpu::Device,
        uniform_bind_group_layout: &wgpu::BindGroupLayout,
        format: wgpu::TextureFormat,
        base_config: &RenderConfig,
        culling_mode: crate::renderer::config::CullingMode,
    ) -> wgpu::RenderPipeline {
        let mut config = base_config.clone();
        config.culling = culling_mode;
        let (pipeline, _) = Self::create_pipeline_with_config(
            device,
            uniform_bind_group_layout,
            format,
            &config,
            1, // Width and height don't matter for pipeline creation when not using multisampling
            1,
        );
        pipeline
    }

    /// Create pipeline with core method name for RenderCore compatibility
    pub fn new_with_config_core(
        device: &wgpu::Device,
        uniform_bind_group_layout: &wgpu::BindGroupLayout,
        format: wgpu::TextureFormat,
        config: RenderConfig,
        width: u32,
        height: u32,
    ) -> Self {
        let (pipeline, multisampled_framebuffer) = Self::create_pipeline_with_config(
            device,
            uniform_bind_group_layout,
            format,
            &config,
            width,
            height,
        );
        Self {
            pipeline,
            config,
            format,
            multisampled_framebuffer,
        }
    }

    /// Get the current texture format used by this pipeline
    pub fn get_format(&self) -> wgpu::TextureFormat {
        self.format
    }
}
