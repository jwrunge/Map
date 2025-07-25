/// Demonstrates wgpu's automatic WebGPU/WebGL fallback for browser compatibility
/// 
/// This shows how your existing code automatically supports:
/// - Modern browsers: WebGPU (high performance)
/// - Older browsers: WebGL (broad compatibility)

use wgpu;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct BackendInfo {
    pub backend_type: String,
    pub adapter_name: String,
    pub driver_info: String,
    pub features: Vec<String>,
    pub limits: BackendLimits,
}

#[derive(Debug, Clone)]
pub struct BackendLimits {
    pub max_texture_dimension_2d: u32,
    pub max_bind_groups: u32,
    pub max_dynamic_uniform_buffers_per_pipeline_layout: u32,
}

impl BackendInfo {
    pub fn from_adapter(adapter: &wgpu::Adapter) -> Self {
        let info = adapter.get_info();
        let features = adapter.features();
        let limits = adapter.limits();
        
        Self {
            backend_type: format!("{:?}", info.backend),
            adapter_name: info.name.clone(),
            driver_info: format!("{} - {}", info.vendor, info.device),
            features: Self::feature_list(features),
            limits: BackendLimits {
                max_texture_dimension_2d: limits.max_texture_dimension_2d,
                max_bind_groups: limits.max_bind_groups,
                max_dynamic_uniform_buffers_per_pipeline_layout: limits.max_dynamic_uniform_buffers_per_pipeline_layout,
            },
        }
    }
    
    fn feature_list(features: wgpu::Features) -> Vec<String> {
        let mut feature_names = Vec::new();
        
        if features.contains(wgpu::Features::DEPTH_CLIP_CONTROL) {
            feature_names.push("DEPTH_CLIP_CONTROL".to_string());
        }
        if features.contains(wgpu::Features::TIMESTAMP_QUERY) {
            feature_names.push("TIMESTAMP_QUERY".to_string());
        }
        if features.contains(wgpu::Features::PIPELINE_STATISTICS_QUERY) {
            feature_names.push("PIPELINE_STATISTICS_QUERY".to_string());
        }
        if features.contains(wgpu::Features::MULTI_DRAW_INDIRECT) {
            feature_names.push("MULTI_DRAW_INDIRECT".to_string());
        }
        if features.contains(wgpu::Features::INDIRECT_FIRST_INSTANCE) {
            feature_names.push("INDIRECT_FIRST_INSTANCE".to_string());
        }
        
        if feature_names.is_empty() {
            feature_names.push("Basic features only".to_string());
        }
        
        feature_names
    }
    
    pub fn is_webgl(&self) -> bool {
        self.backend_type.contains("Gl")
    }
    
    pub fn is_webgpu(&self) -> bool {
        self.backend_type.contains("BrowserWebGpu")
    }
    
    pub fn compatibility_level(&self) -> CompatibilityLevel {
        if self.is_webgpu() {
            CompatibilityLevel::Modern
        } else if self.is_webgl() {
            CompatibilityLevel::Legacy
        } else {
            CompatibilityLevel::Desktop
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompatibilityLevel {
    Desktop,  // Native platforms
    Modern,   // WebGPU in browsers
    Legacy,   // WebGL fallback
}

impl CompatibilityLevel {
    pub fn description(&self) -> &str {
        match self {
            CompatibilityLevel::Desktop => "Native graphics APIs (Vulkan/Metal/DX12)",
            CompatibilityLevel::Modern => "WebGPU - Modern browser support",
            CompatibilityLevel::Legacy => "WebGL - Maximum browser compatibility",
        }
    }
    
    pub fn performance_tier(&self) -> &str {
        match self {
            CompatibilityLevel::Desktop => "🔥 Highest performance",
            CompatibilityLevel::Modern => "⚡ High performance", 
            CompatibilityLevel::Legacy => "✅ Good performance",
        }
    }
}

/// Create a headless context to test backend compatibility
pub async fn detect_graphics_backend() -> Result<BackendInfo> {
    // Use the same backend selection as your main code
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        #[cfg(not(target_arch = "wasm32"))]
        backends: wgpu::Backends::PRIMARY,
        #[cfg(target_arch = "wasm32")]
        backends: wgpu::Backends::BROWSER_WEBGPU | wgpu::Backends::GL,
        flags: wgpu::InstanceFlags::default(),
        ..Default::default()
    });
    
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None, // Headless
            force_fallback_adapter: false,
        })
        .await;
    
    match adapter {
        Some(adapter) => Ok(BackendInfo::from_adapter(&adapter)),
        None => Err(anyhow::anyhow!("Failed to find suitable adapter")),
    }
}

/// Test different backend preferences
pub async fn test_backend_fallback() -> Result<Vec<BackendInfo>> {
    let mut results = Vec::new();
    
    // Test WebGPU first (if available)
    #[cfg(target_arch = "wasm32")]
    {
        if let Ok(info) = test_specific_backend(wgpu::Backends::BROWSER_WEBGPU).await {
            results.push(info);
        }
        
        // Test WebGL fallback
        if let Ok(info) = test_specific_backend(wgpu::Backends::GL).await {
            results.push(info);
        }
    }
    
    // Test native backends
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Ok(info) = test_specific_backend(wgpu::Backends::PRIMARY).await {
            results.push(info);
        }
    }
    
    Ok(results)
}

async fn test_specific_backend(backend: wgpu::Backends) -> Result<BackendInfo> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: backend,
        flags: wgpu::InstanceFlags::default(),
        ..Default::default()
    });
    
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await;
    
    match adapter {
        Some(adapter) => Ok(BackendInfo::from_adapter(&adapter)),
        None => Err(anyhow::anyhow!("Backend not available")),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use pollster;
    
    println!("🎮 Graphics Backend Compatibility Test");
    println!("=====================================");
    println!();
    
    // Test current backend
    match pollster::block_on(detect_graphics_backend()) {
        Ok(info) => {
            println!("✅ Current Backend Detected:");
            println!("   Type: {}", info.backend_type);
            println!("   Adapter: {}", info.adapter_name);
            println!("   Driver: {}", info.driver_info);
            println!("   Compatibility: {} ({})", 
                    info.compatibility_level().description(),
                    info.compatibility_level().performance_tier());
            println!("   Features: {:?}", info.features);
            println!("   Limits:");
            println!("     Max 2D texture: {}px", info.limits.max_texture_dimension_2d);
            println!("     Max bind groups: {}", info.limits.max_bind_groups);
        }
        Err(e) => println!("❌ Failed to detect backend: {}", e),
    }
    
    println!();
    println!("🌐 Browser Compatibility Guide:");
    println!("==============================");
    println!("Your wgpu code will automatically:");
    println!("• Use WebGPU in Chrome 113+, Firefox 131+, Safari 18+");
    println!("• Fall back to WebGL in older browsers");
    println!("• Provide universal browser support back to ~2015");
    println!();
    println!("No code changes needed - wgpu handles the fallback automatically!");
}

// For WASM, we need async main
#[cfg(target_arch = "wasm32")]
fn main() {
    use wasm_bindgen_futures::spawn_local;
    
    spawn_local(async {
        println!("🌐 Browser Graphics Backend Test");
        println!("================================");
        
        match detect_graphics_backend().await {
            Ok(info) => {
                log::info!("Graphics backend: {}", info.backend_type);
                
                if info.is_webgpu() {
                    println!("🚀 Using WebGPU - Modern browser detected!");
                } else if info.is_webgl() {
                    println!("🔄 Using WebGL - Compatibility mode active");
                } else {
                    println!("🖥️ Using native graphics APIs");
                }
                
                println!("Adapter: {}", info.adapter_name);
                println!("Performance: {}", info.compatibility_level().performance_tier());
            }
            Err(e) => {
                log::error!("Failed to detect graphics backend: {}", e);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compatibility_levels() {
        assert_eq!(CompatibilityLevel::Desktop.performance_tier(), "🔥 Highest performance");
        assert_eq!(CompatibilityLevel::Modern.performance_tier(), "⚡ High performance");
        assert_eq!(CompatibilityLevel::Legacy.performance_tier(), "✅ Good performance");
    }
}
