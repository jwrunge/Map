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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_config_default() {
        let config = RenderConfig::default();
        assert_eq!(config.antialiasing, AntialiasingMode::Msaa4x);
        assert_eq!(config.culling, CullingMode::BackfaceCulling);
        assert_eq!(config.alpha_blending, false);
    }

    #[test]
    fn test_render_config_2d() {
        let config = RenderConfig::for_2d();
        assert_eq!(config.antialiasing, AntialiasingMode::Msaa4x);
        assert_eq!(config.culling, CullingMode::None);
        assert_eq!(config.alpha_blending, true);
    }

    #[test]
    fn test_render_config_3d() {
        let config = RenderConfig::for_3d();
        assert_eq!(config.antialiasing, AntialiasingMode::Msaa4x);
        assert_eq!(config.culling, CullingMode::BackfaceCulling);
        assert_eq!(config.alpha_blending, false);
    }

    #[test]
    fn test_render_config_performance() {
        let config = RenderConfig::performance();
        assert_eq!(config.antialiasing, AntialiasingMode::None);
        assert_eq!(config.culling, CullingMode::BackfaceCulling);
        assert_eq!(config.alpha_blending, false);
    }

    #[test]
    fn test_antialiasing_mode_sample_count() {
        assert_eq!(AntialiasingMode::None.sample_count(), 1);
        assert_eq!(AntialiasingMode::Msaa2x.sample_count(), 2);
        assert_eq!(AntialiasingMode::Msaa4x.sample_count(), 4);
        assert_eq!(AntialiasingMode::Msaa8x.sample_count(), 8);
    }

    #[test]
    fn test_antialiasing_mode_is_multisampled() {
        assert_eq!(AntialiasingMode::None.is_multisampled(), false);
        assert_eq!(AntialiasingMode::Msaa2x.is_multisampled(), true);
        assert_eq!(AntialiasingMode::Msaa4x.is_multisampled(), true);
        assert_eq!(AntialiasingMode::Msaa8x.is_multisampled(), true);
    }

    #[test]
    fn test_culling_mode_variants() {
        let none = CullingMode::None;
        let backface = CullingMode::BackfaceCulling;
        let frontface = CullingMode::FrontfaceCulling;
        
        // Test equality
        assert_eq!(none, CullingMode::None);
        assert_eq!(backface, CullingMode::BackfaceCulling);
        assert_eq!(frontface, CullingMode::FrontfaceCulling);
        assert_ne!(none, backface);
        assert_ne!(backface, frontface);
    }

    #[test]
    fn test_culling_mode_to_wgpu() {
        assert_eq!(CullingMode::None.to_wgpu(), None);
        assert_eq!(CullingMode::BackfaceCulling.to_wgpu(), Some(wgpu::Face::Back));
        assert_eq!(CullingMode::FrontfaceCulling.to_wgpu(), Some(wgpu::Face::Front));
    }

    #[test]
    fn test_config_modification() {
        let mut config = RenderConfig::default();
        
        config.antialiasing = AntialiasingMode::None;
        assert_eq!(config.antialiasing, AntialiasingMode::None);
        
        config.culling = CullingMode::None;
        assert_eq!(config.culling, CullingMode::None);
        
        config.alpha_blending = true;
        assert_eq!(config.alpha_blending, true);
    }

    #[test]
    fn test_msaa_compatibility_scenarios() {
        // Test scenarios that caused the original encoder validation error
        let configs = [
            RenderConfig::default(),
            RenderConfig::for_2d(),
            RenderConfig::for_3d(),
            RenderConfig::performance(),
        ];
        
        for config in &configs {
            // Verify sample count is valid
            let sample_count = config.antialiasing.sample_count();
            assert!(sample_count >= 1 && sample_count <= 8);
            assert!(sample_count.is_power_of_two());
        }
    }
}
