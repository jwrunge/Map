//! GPU context and resource management
//!
//! Handles the low-level wgpu setup and resource lifecycle

use winit::window::Window;

/// GPU initialization errors
#[derive(Debug)]
pub enum GpuError {
    SurfaceCreation(String),
    AdapterRequest(String),
    DeviceRequest(String),
}

impl std::fmt::Display for GpuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpuError::SurfaceCreation(msg) => write!(f, "Surface creation failed: {}", msg),
            GpuError::AdapterRequest(msg) => write!(f, "Adapter request failed: {}", msg),
            GpuError::DeviceRequest(msg) => write!(f, "Device request failed: {}", msg),
        }
    }
}

impl std::error::Error for GpuError {}

/// Manages GPU device, surface, and core resources
pub struct GpuContext {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: std::sync::Arc<Window>,
    pub depth_texture: wgpu::Texture,
    pub depth_view: wgpu::TextureView,
}

impl GpuContext {
    pub async fn new(window: std::sync::Arc<Window>) -> Result<Self, GpuError> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::BROWSER_WEBGPU | wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone())
            .map_err(|e| GpuError::SurfaceCreation(format!("{:?}", e)))?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_err(|e| GpuError::AdapterRequest(format!("{:?}", e)))?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                ..Default::default()
            })
            .await
            .map_err(|e| GpuError::DeviceRequest(format!("{:?}", e)))?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = {
            // On web platforms, prefer BGRA8Unorm for better compatibility
            #[cfg(target_arch = "wasm32")]
            {
                surface_caps
                    .formats
                    .iter()
                    .find(|f| **f == wgpu::TextureFormat::Bgra8Unorm)
                    .copied()
                    .unwrap_or_else(|| {
                        surface_caps
                            .formats
                            .iter()
                            .find(|f| f.is_srgb())
                            .copied()
                            .unwrap_or(surface_caps.formats[0])
                    })
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                surface_caps
                    .formats
                    .iter()
                    .find(|f| f.is_srgb())
                    .copied()
                    .unwrap_or(surface_caps.formats[0])
            }
        };

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let (depth_texture, depth_view) = Self::create_depth_texture(&device, &config);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            depth_texture,
            depth_view,
        })
    }

    fn create_depth_texture(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> (wgpu::Texture, wgpu::TextureView) {
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some("depth_texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            
            // Recreate depth texture with new size
            let (depth_texture, depth_view) = Self::create_depth_texture(&self.device, &self.config);
            self.depth_texture = depth_texture;
            self.depth_view = depth_view;
        }
    }
}
