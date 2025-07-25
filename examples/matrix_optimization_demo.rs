use glam::{Vec3, Mat4};
use map::*;
use std::time::Instant;

/// Demonstrates the performance improvement from matrix caching optimization
fn main() {
    let mut triangles: Vec<Triangle> = Vec::new();
    
    // Create 1000 triangles for performance testing
    for i in 0..1000 {
        let mut triangle = Triangle::new();
        triangle.get_transform_mut().set_position(Vec3::new(i as f32, 0.0, 0.0));
        triangles.push(triangle);
    }
    
    println!("Matrix Optimization Performance Demo");
    println!("=====================================");
    println!("Testing with {} triangles", triangles.len());
    
    // Test uncached matrix computation (fallback method)
    let start = Instant::now();
    let mut uncached_total = 0;
    for _ in 0..10000 {
        for triangle in &triangles {
            let _matrix = triangle.get_matrix(); // Uses uncached fallback
            uncached_total += 1;
        }
    }
    let uncached_time = start.elapsed();
    
    // Test cached matrix computation
    let start = Instant::now();
    let mut cached_total = 0;
    for _ in 0..10000 {
        for triangle in &mut triangles {
            let _matrix = triangle.get_matrix_cached(); // Uses cached method
            cached_total += 1;
        }
    }
    let cached_time = start.elapsed();
    
    println!();
    println!("Results:");
    println!("--------");
    println!("Uncached computation: {:?} ({} operations)", uncached_time, uncached_total);
    println!("Cached computation:   {:?} ({} operations)", cached_time, cached_total);
    
    if cached_time < uncached_time {
        let speedup = uncached_time.as_nanos() as f64 / cached_time.as_nanos() as f64;
        println!("Speedup: {:.2}x faster with caching", speedup);
    } else {
        println!("Note: For static objects, the speedup is most apparent after the first computation");
    }
    
    // Demonstrate memory usage optimization
    println!();
    println!("Memory Usage Analysis:");
    println!("---------------------");
    println!("Transform size: {} bytes", std::mem::size_of::<Transform>());
    println!("  - Vec3 position: {} bytes", std::mem::size_of::<Vec3>());
    println!("  - Quat rotation: {} bytes", std::mem::size_of::<glam::Quat>());
    println!("  - Vec3 scale:    {} bytes", std::mem::size_of::<Vec3>());
    println!("  - Option<Mat4>:  {} bytes", std::mem::size_of::<Option<Mat4>>());
    println!("  - bool dirty:    {} bytes", std::mem::size_of::<bool>());
    println!();
    println!("Total per object: {} bytes transform + 64 bytes cached matrix = {} bytes",
             std::mem::size_of::<Transform>(),
             std::mem::size_of::<Transform>() + 64);
    println!("Without caching:  40 bytes transform + repeated 64-byte computations");
    println!();
    println!("For {} objects:", triangles.len());
    println!("  With caching:    {} KB total", 
             (triangles.len() * std::mem::size_of::<Transform>()) / 1024);
    println!("  Matrix storage:  {} KB additional",
             (triangles.len() * 64) / 1024);
    
    // Demonstrate actual rendering scenario
    println!();
    println!("Rendering Scenario Test:");
    println!("------------------------");
    
    let camera = Camera::new(1.0);
    let view_proj = camera.get_view_projection_matrix();
    
    // Simulate multiple frames with static objects
    let start = Instant::now();
    for frame in 0..100 {
        for triangle in &mut triangles {
            // First access computes and caches
            let object_matrix = triangle.get_matrix_cached();
            let _final_matrix = view_proj * object_matrix;
            
            // Subsequent accesses in same frame use cache
            let _object_matrix2 = triangle.get_matrix_cached();
            let _final_matrix2 = view_proj * _object_matrix2;
        }
        
        // Only every 10th frame do we modify some objects
        if frame % 10 == 0 {
            for (i, triangle) in triangles.iter_mut().enumerate() {
                if i % 100 == 0 { // Only 1% of objects move
                    triangle.get_transform_mut().translate_xyz(0.01, 0.0, 0.0);
                }
            }
        }
    }
    let render_time = start.elapsed();
    
    println!("100 frames with 2 matrix accesses per object: {:?}", render_time);
    println!("  - 99% static objects benefit from caching");
    println!("  - 1% dynamic objects recompute when dirty");
}
