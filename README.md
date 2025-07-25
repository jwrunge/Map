# Map Graphics Library

A flexible 3D graphics library built with wgpu that can run in both windowed and headless modes.

## Features

-   **Windowed Mode**: Full window management with winit for development and standalone applications
-   **Headless Mode**: Render to textures/buffers for embedded use in existing applications
-   **WASM Support**: Run in web browsers with WebGL/WebGPU backends
-   **3D Primitives**: Triangles, quads, and cubes with GPU-accelerated transforms
-   **Performance Optimized**: Vertex buffer caching, dynamic uniform buffers, and batch rendering
-   **Antialiasing**: Configurable MSAA (None, 2x, 4x, 8x) with device capability detection
-   **Culling Control**: Configurable backface culling for 2D vs 3D rendering optimization
-   **Render Presets**: Optimized configurations for 2D, 3D, and performance modes

## Usage

### Windowed Mode (Development)

```rust
// Run with default windowing feature
use map;

fn main() {
    map::run(); // Opens a window and runs the demo
}
```

### Headless Mode (Embedded)

```rust
// Build with: cargo build --no-default-features --features headless
use map::{HeadlessRenderer, Scene};

#[async_std::main] // or tokio::main
async fn main() -> anyhow::Result<()> {
    let mut renderer = HeadlessRenderer::new(800, 600).await?;
    let mut scene = Scene::new();

    // Add objects to scene
    scene.create_triangle(1.0);

    // Render to pixel buffer
    let (triangles, quads, cubes) = scene.get_all_renderables();
    let pixels = renderer.render_to_buffer(&triangles, &quads, &cubes)?;

    // Use pixels data (save to file, send over network, etc.)
    Ok(())
}
```

### WASM (Web)

```bash
# Build for web
./build-wasm.sh
# Serve at http://localhost:8000
python3 -m http.server 8000
```

## Configuration

The library provides extensive rendering configuration options for both windowed and headless modes.

### Antialiasing

```rust
use map::{HeadlessRenderer, AntialiasingMode};

let mut renderer = HeadlessRenderer::new(800, 600).await?;

// Set specific antialiasing mode
renderer.set_antialiasing(AntialiasingMode::Msaa4x)?;

// Available modes: None, Msaa2x, Msaa4x, Msaa8x
// (device support varies - 4x is guaranteed by WebGPU spec)
```

### Backface Culling

```rust
use map::{HeadlessRenderer, CullingMode};

// For 2D objects (triangles, quads) - render both sides
renderer.set_culling(CullingMode::None)?;

// For 3D objects (cubes) - cull back faces for performance
renderer.set_culling(CullingMode::BackfaceCulling)?;
```

### Preset Configurations

```rust
// Optimized for 2D rendering (no culling, alpha blending)
renderer.set_2d_mode()?;

// Optimized for 3D rendering (backface culling, no alpha blending)
renderer.set_3d_mode()?;

// Maximum performance (no antialiasing, minimal overhead)
renderer.set_performance_mode()?;
```

### Custom Configuration

```rust
use map::{RenderConfig, AntialiasingMode, CullingMode};

let config = RenderConfig {
    antialiasing: AntialiasingMode::Msaa4x,
    culling: CullingMode::None,
    alpha_blending: true,
};
renderer.update_config(config)?;
```

### Device Capability Detection

```rust
// Query supported antialiasing modes for the current device
let supported = AntialiasingMode::get_supported_modes(
    renderer.device(),
    wgpu::TextureFormat::Rgba8UnormSrgb
);
println!("Supported: {:?}", supported);

// Get the best supported mode automatically
let best = AntialiasingMode::get_best_supported(
    renderer.device(),
    wgpu::TextureFormat::Rgba8UnormSrgb
);
```

## Build Features

-   `windowing` (default): Includes winit window management
-   `headless`: Enables headless rendering (requires no windowing)

## Dependencies

-   **wgpu**: Cross-platform graphics API
-   **glam**: Mathematics library for 3D transformations
-   **winit**: Window management (optional, windowing feature only)
-   **web-time**: WASM-compatible timing

## Architecture

```
┌─────────────────┐  ┌──────────────────┐
│ Windowed App    │  │ Headless App     │
├─────────────────┤  ├──────────────────┤
│ Renderer        │  │ HeadlessRenderer │
├─────────────────┤  ├──────────────────┤
│        Core Rendering System          │
├─────────────────────────────────────────┤
│ Scene │ Renderables │ GPU Pipeline    │
└─────────────────────────────────────────┘
```

## License

[Add your license here]
