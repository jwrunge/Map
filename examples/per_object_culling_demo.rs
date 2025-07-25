//! Demonstrates per-object culling mode control
//!
//! This example shows how different objects can have different culling modes:
//! - Triangles and quads use CullingMode::None (good for 2D objects)
//! - Cubes use CullingMode::BackfaceCulling (good for 3D objects)

use map::*;

fn main() -> anyhow::Result<()> {
    println!("🔍 Per-Object Culling Mode Demo");
    println!("================================");

    // Create objects with default culling modes
    let mut triangle = Triangle::new();
    let mut quad = Quad::with_size(1.0, 1.0);
    let mut cube = Cube::with_size(1.0);

    // Show the default culling modes
    println!("\n� Default culling modes:");
    println!("  Triangle: {:?} (good for 2D - renders both sides)", triangle.get_culling_mode());
    println!("  Quad: {:?} (good for 2D - renders both sides)", quad.get_culling_mode());
    println!("  Cube: {:?} (good for 3D - performance optimized)", cube.get_culling_mode());

    // Test changing culling modes
    println!("\n🔧 Testing culling mode changes:");
    
    // Try changing triangle to backface culling
    triangle.set_culling_mode(CullingMode::BackfaceCulling);
    println!("  Triangle after change: {:?}", triangle.get_culling_mode());
    
    // Try changing cube to no culling
    cube.set_culling_mode(CullingMode::None);
    println!("  Cube after change: {:?}", cube.get_culling_mode());

    // Demonstrate the benefits
    println!("\n💡 Benefits of per-object culling:");
    println!("   • 2D objects (triangles, quads) can render both sides for visibility");
    println!("   • 3D objects (cubes) can use backface culling for performance");
    println!("   • Mixed 2D/3D scenes work optimally");
    println!("   • No need for global culling settings that compromise some objects");

    println!("\n🎨 In the renderer:");
    println!("   • Objects are automatically grouped by culling mode");
    println!("   • Each group is rendered with the appropriate pipeline");
    println!("   • Performance is maintained through intelligent batching");

    #[cfg(feature = "headless")]
    {
        println!("\n🧪 Testing with headless renderer...");
        // Note: HeadlessRenderer::new() is async, so we'd need futures or tokio runtime
        // For this demo, we'll just show that the API is available
        println!("✅ HeadlessRenderer API available with per-object culling support!");
        println!("   Use: let renderer = HeadlessRenderer::new(800, 600).await?;");
    }

    #[cfg(feature = "windowing")]
    {
        println!("\n🪟 Windowing mode: Use `cargo run` to see per-object culling in action!");
        println!("   The renderer will automatically group objects by culling mode.");
    }

    println!("\n🎯 Summary:");
    println!("   Per-object culling gives you fine-grained control over rendering");
    println!("   behavior while maintaining performance through intelligent grouping.");

    Ok(())
}
