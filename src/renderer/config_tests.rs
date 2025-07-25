//! Tests for rendering configuration

#[cfg(test)]
mod tests {
    use super::super::{RenderConfig, AntialiasingMode, CullingMode};

    #[test]
    fn test_default_config() {
        let config = RenderConfig::default();
        
        assert_eq!(config.antialiasing, AntialiasingMode::Msaa4x);
        assert_eq!(config.culling, CullingMode::BackfaceCulling);
        assert_eq!(config.alpha_blending, false);
    }

    #[test]
    fn test_2d_config() {
        let config = RenderConfig::for_2d();
        
        // 2D should have no culling for double-sided rendering
        assert_eq!(config.culling, CullingMode::None);
        assert_eq!(config.alpha_blending, true);
        assert_eq!(config.antialiasing, AntialiasingMode::Msaa4x);
    }

    #[test]
    fn test_3d_config() {
        let config = RenderConfig::for_3d();
        
        // 3D should use backface culling for performance
        assert_eq!(config.culling, CullingMode::BackfaceCulling);
        assert_eq!(config.alpha_blending, false);
        assert_eq!(config.antialiasing, AntialiasingMode::Msaa4x);
    }

    #[test]
    fn test_performance_config() {
        let config = RenderConfig::performance();
        
        // Performance mode should disable MSAA
        assert_eq!(config.antialiasing, AntialiasingMode::None);
        assert_eq!(config.culling, CullingMode::BackfaceCulling);
        assert_eq!(config.alpha_blending, false);
    }

    #[test]
    fn test_antialiasing_sample_counts() {
        assert_eq!(AntialiasingMode::None.sample_count(), 1);
        assert_eq!(AntialiasingMode::Msaa2x.sample_count(), 2);
        assert_eq!(AntialiasingMode::Msaa4x.sample_count(), 4);
        assert_eq!(AntialiasingMode::Msaa8x.sample_count(), 8);
    }

    #[test]
    fn test_antialiasing_multisampled_detection() {
        assert!(!AntialiasingMode::None.is_multisampled());
        assert!(AntialiasingMode::Msaa2x.is_multisampled());
        assert!(AntialiasingMode::Msaa4x.is_multisampled());
        assert!(AntialiasingMode::Msaa8x.is_multisampled());
    }

    #[test]
    fn test_culling_mode_to_wgpu() {
        assert_eq!(CullingMode::None.to_wgpu(), None);
        assert_eq!(CullingMode::BackfaceCulling.to_wgpu(), Some(wgpu::Face::Back));
        assert_eq!(CullingMode::FrontfaceCulling.to_wgpu(), Some(wgpu::Face::Front));
    }

    #[test]
    fn test_config_clone_and_modify() {
        let mut config = RenderConfig::default();
        let original_culling = config.culling;
        
        let mut modified_config = config.clone();
        modified_config.culling = CullingMode::None;
        
        // Original should be unchanged
        assert_eq!(config.culling, original_culling);
        assert_eq!(modified_config.culling, CullingMode::None);
    }

    #[test]
    fn test_msaa_compatibility_checks() {
        let msaa_config = RenderConfig::default(); // Has MSAA4x
        let no_msaa_config = RenderConfig::performance(); // No MSAA
        
        // Verify they have different sample counts
        assert_ne!(
            msaa_config.antialiasing.sample_count(),
            no_msaa_config.antialiasing.sample_count()
        );
        
        // Verify the non-MSAA config has sample count 1
        assert_eq!(no_msaa_config.antialiasing.sample_count(), 1);
        
        // Verify MSAA config has sample count > 1
        assert!(msaa_config.antialiasing.sample_count() > 1);
    }
}
