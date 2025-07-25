//! Example of using the headless renderer
//! 
//! This shows how to use the Map library in embedded mode without windowing.

#[cfg(feature = "headless")]
use map::{HeadlessRenderer, Scene};
#[cfg(feature = "headless")]
use anyhow::Result;

#[cfg(feature = "headless")]
fn main() -> Result<()> {
    // This would work with an async runtime in a real application
    println!("Headless rendering example:");
    println!("1. Create HeadlessRenderer::new(800, 600).await");
    println!("2. Create Scene::new() and add objects");
    println!("3. Call renderer.render_to_buffer(&triangles, &quads, &cubes)");
    println!("4. Process the returned pixel buffer");
    println!();
    println!("Build with: cargo build --no-default-features --features headless");
    println!("This eliminates winit dependency for embedded use.");
    Ok(())
}

#[cfg(not(feature = "headless"))]
fn main() {
    println!("This example requires the 'headless' feature.");
    println!("Run with: cargo run --example headless --no-default-features --features headless");
}
