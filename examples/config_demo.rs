//! Comprehensive example demonstrating antialiasing and culling configuration
//!
//! This example shows how to:
//! 1. Toggle between different antialiasing modes
//! 2. Configure backface culling for 2D vs 3D rendering
//! 3. Switch between optimized presets

use map::*;

#[cfg(feature = "headless")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Example 1: Basic headless rendering with default settings
    println!("🎨 Creating headless renderer with default settings...");
    let mut renderer = HeadlessRenderer::new(800, 600).await?;
    
    // Create some test objects
    let mut scene = Scene::new();
    let triangle_id = scene.create_triangle(0.3);
    let _quad_id = scene.create_quad(0.5);
    let _cube_id = scene.create_cube(0.4);
    
    // Position objects
    if let Some(triangle) = scene.get_triangle_mut(triangle_id) {
        triangle.get_transform_mut().position.x = -2.0;
    }
    // Note: Scene doesn't have get_cube_mut, so we'll leave the cube in default position
    
    // Helper function to render the scene
    let render_scene = |renderer: &mut HeadlessRenderer, scene: &Scene| -> anyhow::Result<()> {
        let (triangles, quads, cubes) = scene.get_all_renderables();
        let triangles: Vec<Triangle> = triangles.into_iter().cloned().collect();
        let quads: Vec<Quad> = quads.into_iter().cloned().collect();
        let cubes: Vec<Cube> = cubes.into_iter().cloned().collect();
        renderer.render_to_buffer(&triangles, &quads, &cubes)?;
        Ok(())
    };
    
    // Example 2: Configure different antialiasing modes
    println!("📐 Testing different antialiasing modes...");
    
    // Query supported modes
    let supported_modes = AntialiasingMode::get_supported_modes(renderer.device(), wgpu::TextureFormat::Rgba8UnormSrgb);
    println!("  📋 Supported modes on this device: {:?}", supported_modes);
    
    // No antialiasing (fastest)
    renderer.set_antialiasing(AntialiasingMode::None)?;
    println!("  ✓ Rendering with no antialiasing");
    render_scene(&mut renderer, &scene)?;
    
    // 4x MSAA (good balance) - guaranteed by WebGPU spec
    match renderer.set_antialiasing(AntialiasingMode::Msaa4x) {
        Ok(_) => {
            println!("  ✓ Rendering with 4x MSAA");
            render_scene(&mut renderer, &scene)?;
        }
        Err(e) => println!("  ⚠ 4x MSAA failed: {}", e),
    }
    
    // Example 3: Configure culling modes
    println!("🎭 Testing different culling modes...");
    
    // No culling (good for 2D objects like triangles and quads)
    renderer.set_culling(CullingMode::None)?;
    println!("  ✓ Rendering with no backface culling (2D mode)");
    render_scene(&mut renderer, &scene)?;
    
    // Backface culling (good for 3D objects like cubes)
    renderer.set_culling(CullingMode::BackfaceCulling)?;
    println!("  ✓ Rendering with backface culling (3D mode)");
    render_scene(&mut renderer, &scene)?;
    
    // Example 4: Use convenient presets
    println!("⚙️  Testing preset configurations...");
    
    // 2D optimized settings
    renderer.set_2d_mode()?;
    println!("  ✓ 2D mode: no culling, alpha blending enabled");
    render_scene(&mut renderer, &scene)?;
    
    // 3D optimized settings
    renderer.set_3d_mode()?;
    println!("  ✓ 3D mode: backface culling, no alpha blending");
    render_scene(&mut renderer, &scene)?;
    
    // Performance mode
    renderer.set_performance_mode()?;
    println!("  ✓ Performance mode: no antialiasing, minimal overhead");
    render_scene(&mut renderer, &scene)?;
    
    // Example 5: Custom configuration
    println!("🔧 Creating custom configuration...");
    let custom_config = RenderConfig {
        antialiasing: AntialiasingMode::Msaa4x, // Use 4x instead of 2x for compatibility
        culling: CullingMode::None,
        alpha_blending: true,
    };
    renderer.update_config(custom_config)?;
    println!("  ✓ Custom: 4x MSAA, no culling, alpha blending");
    render_scene(&mut renderer, &scene)?;
    
    // Show current configuration
    let config = renderer.get_config();
    println!("📊 Final configuration:");
    println!("  • Antialiasing: {:?} ({}x samples)", config.antialiasing, config.antialiasing.sample_count());
    println!("  • Culling: {:?}", config.culling);
    println!("  • Alpha blending: {}", config.alpha_blending);
    
    println!("🎉 All configuration tests completed successfully!");
    Ok(())
}

#[cfg(feature = "windowing")]
fn main() {
    println!("This example requires the headless feature.");
    println!("Run with: cargo run --example config_demo --no-default-features --features headless");
}

#[cfg(not(any(feature = "windowing", feature = "headless")))]
fn main() {
    println!("No features enabled. Please enable either 'windowing' or 'headless' feature.");
}
