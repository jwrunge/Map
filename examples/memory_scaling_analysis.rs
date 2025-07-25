use glam::{Vec3, Mat4};
use map::*;
use std::time::Instant;

/// Analyzes memory usage and performance scaling for large entity counts
fn main() {
    println!("Memory Scaling Analysis for Transform Caching");
    println!("============================================");
    
    // Test different entity counts
    let entity_counts = vec![1_000, 10_000, 100_000, 500_000, 1_000_000];
    
    for &count in &entity_counts {
        println!("\n📊 Testing with {} entities:", count);
        println!("{}", "=".repeat(40));
        
        // Memory calculations
        let transform_size = std::mem::size_of::<Transform>();
        let cached_overhead = 80; // Option<Mat4> size
        let base_transform = 48; // Original transform size
        
        let total_memory_with_cache = count * transform_size;
        let cache_memory_only = count * cached_overhead;
        let total_memory_without_cache = count * base_transform;
        
        println!("Memory Usage:");
        println!("  Without caching: {} MB", bytes_to_mb(total_memory_without_cache));
        println!("  With caching:    {} MB", bytes_to_mb(total_memory_with_cache));
        println!("  Cache overhead:  {} MB ({:.1}% increase)", 
                bytes_to_mb(cache_memory_only),
                (cache_memory_only as f64 / total_memory_without_cache as f64) * 100.0);
        
        // Performance simulation
        if count <= 100_000 { // Only test performance for reasonable counts
            test_performance_scaling(count);
        } else {
            println!("Performance: Skipped (too many entities for demo)");
        }
        
        // Memory pressure analysis
        analyze_memory_pressure(count, cache_memory_only);
    }
    
    println!("\n🎯 Recommendations:");
    println!("==================");
    println!("✅ Small games (< 10K entities):     Cache is excellent");
    println!("✅ Medium games (10K-100K entities): Cache is beneficial"); 
    println!("⚠️  Large games (100K-500K entities): Consider selective caching");
    println!("❌ Massive games (> 1M entities):    Cache may cause memory pressure");
    
    println!("\n💡 Optimization Strategies for Large Scale:");
    println!("==========================================");
    println!("1. **Selective Caching**: Only cache frequently-accessed transforms");
    println!("2. **Lazy Caching**: Cache on first access, expire after time");
    println!("3. **Level-of-Detail**: Different transform precision for distant objects");
    println!("4. **Spatial Partitioning**: Only cache transforms in active chunks");
    println!("5. **Static/Dynamic Split**: Only cache static objects");
}

fn bytes_to_mb(bytes: usize) -> f64 {
    bytes as f64 / 1_048_576.0
}

fn test_performance_scaling(entity_count: usize) {
    let mut triangles: Vec<Triangle> = Vec::new();
    
    // Create entities
    let creation_start = Instant::now();
    for i in 0..entity_count {
        let mut triangle = Triangle::new();
        triangle.get_transform_mut().set_position(Vec3::new(
            (i % 1000) as f32, 
            ((i / 1000) % 1000) as f32, 
            (i / 1_000_000) as f32
        ));
        triangles.push(triangle);
    }
    let creation_time = creation_start.elapsed();
    
    // Test matrix access performance
    let access_start = Instant::now();
    let mut checksum = 0u64;
    for triangle in &mut triangles {
        let matrix = triangle.get_matrix_cached();
        // Use matrix to prevent optimization
        checksum = checksum.wrapping_add(matrix.to_cols_array()[0] as u64);
    }
    let access_time = access_start.elapsed();
    
    println!("Performance:");
    println!("  Entity creation: {:?}", creation_time);
    println!("  Matrix access:   {:?} ({} entities/ms)", 
            access_time, 
            entity_count as f64 / access_time.as_millis() as f64);
    println!("  Checksum: {} (prevents optimization)", checksum);
}

fn analyze_memory_pressure(entity_count: usize, cache_memory: usize) {
    let cache_mb = bytes_to_mb(cache_memory);
    
    println!("Memory Pressure Analysis:");
    
    if cache_mb < 10.0 {
        println!("  🟢 LOW: {} MB cache fits easily in most systems", cache_mb);
    } else if cache_mb < 100.0 {
        println!("  🟡 MODERATE: {} MB cache noticeable but manageable", cache_mb);
    } else if cache_mb < 500.0 {
        println!("  🟠 HIGH: {} MB cache significant memory usage", cache_mb);
    } else {
        println!("  🔴 CRITICAL: {} MB cache may cause performance issues", cache_mb);
        
        // Suggest alternatives
        println!("  Alternatives:");
        println!("    - Selective caching (only static/visible objects)");
        println!("    - Temporal caching (expire after N frames)");
        println!("    - Compressed transforms (lower precision)");
    }
    
    // Context for different platforms
    if cache_mb > 100.0 {
        println!("  Platform Considerations:");
        println!("    - Mobile (2-4GB RAM): May struggle");
        println!("    - Desktop (8-16GB RAM): Manageable");
        println!("    - Server (32GB+ RAM): No problem");
    }
}
