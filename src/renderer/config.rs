//! Rendering configuration options
//!
//! Provides various settings for controlling rendering behavior

/// Antialiasing settings for the renderer
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AntialiasingMode {
    /// No antialiasing (fastest)
    None,
    /// 2x multisampling
    Msaa2x,
    /// 4x multisampling (good quality/performance balance)
    Msaa4x,
    /// 8x multisampling (highest quality, may be slower)
    Msaa8x,
}

impl AntialiasingMode {
    /// Get the sample count for this antialiasing mode
    pub fn sample_count(&self) -> u32 {
        match self {
            AntialiasingMode::None => 1,
            AntialiasingMode::Msaa2x => 2,
            AntialiasingMode::Msaa4x => 4,
            AntialiasingMode::Msaa8x => 8,
        }
    }

    /// Check if this mode requires multisampling
    pub fn is_multisampled(&self) -> bool {
        self.sample_count() > 1
    }

    /// Validate if this antialiasing mode is supported by the device for the given format
    pub fn is_supported(&self, _device: &wgpu::Device, _format: wgpu::TextureFormat) -> bool {
        if !self.is_multisampled() {
            return true; // Single sampling is always supported
        }

        // Check if the device supports this sample count for the format
        // We'll try to create a test texture descriptor and see if it would be valid
        let sample_count = self.sample_count();

        // For now, we'll use a conservative approach
        // Most devices support 1, 2, and 4 samples. 8 samples is less common.
        match sample_count {
            1 => true,  // Always supported
            2 => true,  // Usually supported
            4 => true,  // WebGPU spec guarantees this
            8 => false, // Not guaranteed, device-specific
            _ => false,
        }
    }

    /// Get the highest supported antialiasing mode for the device (conservative)
    pub fn get_best_supported(_device: &wgpu::Device, _format: wgpu::TextureFormat) -> Self {
        // For now, return 4x MSAA as it's guaranteed by WebGPU spec
        AntialiasingMode::Msaa4x
    }

    /// Get all supported antialiasing modes for the device (conservative)
    pub fn get_supported_modes(_device: &wgpu::Device, _format: wgpu::TextureFormat) -> Vec<Self> {
        vec![
            AntialiasingMode::None,
            AntialiasingMode::Msaa2x,
            AntialiasingMode::Msaa4x,
            // Skip 8x for now as it's not universally supported
        ]
    }
}

/// Culling mode for controlling which faces are rendered
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CullingMode {
    /// Render both front and back faces (good for 2D objects)
    None,
    /// Only render front faces (good for 3D objects)
    BackfaceCulling,
    /// Only render back faces (rarely used)
    FrontfaceCulling,
}

impl CullingMode {
    /// Convert to wgpu cull mode
    pub fn to_wgpu(&self) -> Option<wgpu::Face> {
        match self {
            CullingMode::None => None,
            CullingMode::BackfaceCulling => Some(wgpu::Face::Back),
            CullingMode::FrontfaceCulling => Some(wgpu::Face::Front),
        }
    }
}

/// Complete rendering configuration
#[derive(Debug, Clone)]
pub struct RenderConfig {
    /// Antialiasing mode
    pub antialiasing: AntialiasingMode,
    /// Face culling mode
    pub culling: CullingMode,
    /// Whether to use alpha blending
    pub alpha_blending: bool,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            antialiasing: AntialiasingMode::Msaa4x,
            culling: CullingMode::BackfaceCulling,
            alpha_blending: false,
        }
    }
}

impl RenderConfig {
    /// Create a new config optimized for 2D rendering
    pub fn for_2d() -> Self {
        Self {
            antialiasing: AntialiasingMode::Msaa4x,
            culling: CullingMode::None, // Render both sides for 2D objects
            alpha_blending: true,
        }
    }

    /// Create a new config optimized for 3D rendering
    pub fn for_3d() -> Self {
        Self {
            antialiasing: AntialiasingMode::Msaa4x,
            culling: CullingMode::BackfaceCulling, // Cull back faces for performance
            alpha_blending: false,
        }
    }

    /// Create a config with no antialiasing for maximum performance
    pub fn performance() -> Self {
        Self {
            antialiasing: AntialiasingMode::None,
            culling: CullingMode::BackfaceCulling,
            alpha_blending: false,
        }
    }
}
