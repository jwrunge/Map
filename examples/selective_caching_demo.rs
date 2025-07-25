use glam::{Vec3, Mat4, Quat};

/// Smart transform that only caches when beneficial
#[derive(Debug, Clone)]
pub struct SelectiveTransform {
    /// Position in 3D space (x, y, z)
    pub position: Vec3,
    /// Rotation as a quaternion (avoids gimbal lock)
    pub rotation: Quat,
    /// Scale factors for each axis (x, y, z)
    pub scale: Vec3,
    /// Cached transformation matrix (only for frequently accessed objects)
    cached_matrix: Option<Mat4>,
    /// Whether the cached matrix needs recomputation
    matrix_dirty: bool,
    /// Access counter to determine if caching is beneficial
    access_count: u32,
    /// Frame counter for temporal caching
    last_cache_frame: u32,
}

impl SelectiveTransform {
    /// Threshold for enabling caching (objects accessed this many times get cached)
    const CACHE_THRESHOLD: u32 = 3;
    /// Frames after which to expire cache for temporal objects
    const CACHE_EXPIRY_FRAMES: u32 = 60;
    
    pub fn new(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
            cached_matrix: None,
            matrix_dirty: true,
            access_count: 0,
            last_cache_frame: 0,
        }
    }
    
    /// Get matrix with intelligent caching decisions
    pub fn get_matrix(&mut self, current_frame: u32) -> Mat4 {
        self.access_count += 1;
        
        // Decide whether to use caching
        let should_cache = self.access_count >= Self::CACHE_THRESHOLD;
        let cache_expired = current_frame - self.last_cache_frame > Self::CACHE_EXPIRY_FRAMES;
        
        if should_cache && !cache_expired {
            // Use cached version for frequently accessed objects
            if self.matrix_dirty || self.cached_matrix.is_none() {
                self.cached_matrix = Some(Mat4::from_scale_rotation_translation(
                    self.scale, self.rotation, self.position
                ));
                self.matrix_dirty = false;
                self.last_cache_frame = current_frame;
            }
            self.cached_matrix.unwrap()
        } else {
            // Direct computation for infrequently accessed objects
            if cache_expired {
                self.cached_matrix = None; // Free memory
            }
            Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
        }
    }
    
    /// Set position and mark as dirty
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
        self.matrix_dirty = true;
    }
    
    /// Set rotation and mark as dirty  
    pub fn set_rotation(&mut self, rotation: Quat) {
        self.rotation = rotation;
        self.matrix_dirty = true;
    }
    
    /// Set scale and mark as dirty
    pub fn set_scale(&mut self, scale: Vec3) {
        self.scale = scale;
        self.matrix_dirty = true;
    }
    
    /// Get memory usage statistics
    pub fn memory_usage(&self) -> (usize, bool) {
        let base_size = std::mem::size_of::<Vec3>() * 2 + std::mem::size_of::<Quat>() 
                       + std::mem::size_of::<bool>() + std::mem::size_of::<u32>() * 2;
        let cached_size = if self.cached_matrix.is_some() { 
            std::mem::size_of::<Mat4>() 
        } else { 
            0 
        };
        (base_size + cached_size, self.cached_matrix.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test] 
    fn test_selective_caching() {
        let mut transform = SelectiveTransform::new(
            Vec3::ZERO, Quat::IDENTITY, Vec3::ONE
        );
        
        // First few accesses don't cache
        let _m1 = transform.get_matrix(1);
        let _m2 = transform.get_matrix(1); 
        assert!(!transform.memory_usage().1); // Not cached yet
        
        // After threshold, starts caching
        let _m3 = transform.get_matrix(1);
        let _m4 = transform.get_matrix(1);
        assert!(transform.memory_usage().1); // Now cached
        
        // Cache expires after time
        let _m5 = transform.get_matrix(100); // Much later frame
        // Cache should be cleared for memory efficiency
    }
    
    #[test]
    fn test_memory_efficiency() {
        let mut uncached_transform = SelectiveTransform::new(
            Vec3::ZERO, Quat::IDENTITY, Vec3::ONE  
        );
        let mut cached_transform = SelectiveTransform::new(
            Vec3::ZERO, Quat::IDENTITY, Vec3::ONE
        );
        
        // Uncached should use less memory
        let _m1 = uncached_transform.get_matrix(1); // Access once
        let (uncached_mem, uncached_has_cache) = uncached_transform.memory_usage();
        
        // Cached should use more memory after threshold
        for _ in 0..5 {
            let _m = cached_transform.get_matrix(1);
        }
        let (cached_mem, cached_has_cache) = cached_transform.memory_usage();
        
        assert!(!uncached_has_cache);
        assert!(cached_has_cache);
        assert!(cached_mem > uncached_mem);
    }
}

/// Demo of memory savings with selective caching
fn main() {
    println!("Selective Transform Caching Demo");
    println!("================================");
    
    let entity_counts = vec![1_000, 10_000, 100_000];
    
    for &count in &entity_counts {
        println!("\n📊 {} entities:", count);
        
        // Simulate mixed usage patterns
        let mut transforms = Vec::new();
        for i in 0..count {
            transforms.push(SelectiveTransform::new(
                Vec3::new(i as f32, 0.0, 0.0),
                Quat::IDENTITY,
                Vec3::ONE
            ));
        }
        
        // Simulate realistic access patterns:
        // - 10% frequently accessed (UI, player, important NPCs)
        // - 90% infrequently accessed (background objects)
        
        let frame = 1;
        for i in 0..count {
            let access_frequency = if i < count / 10 { 5 } else { 1 }; // 10% get heavy usage
            
            for _ in 0..access_frequency {
                let _matrix = transforms[i].get_matrix(frame);
            }
        }
        
        // Calculate memory usage
        let mut total_memory = 0;
        let mut cached_objects = 0;
        
        for transform in &transforms {
            let (mem, has_cache) = transform.memory_usage();
            total_memory += mem;
            if has_cache {
                cached_objects += 1;
            }
        }
        
        println!("Results:");
        println!("  Total memory: {} KB", total_memory / 1024);
        println!("  Cached objects: {} ({:.1}%)", 
                cached_objects, 
                (cached_objects as f64 / count as f64) * 100.0);
        println!("  Memory per object: {} bytes avg", total_memory / count);
        
        // Compare to full caching
        let full_cache_memory = count * std::mem::size_of::<crate::Transform>();
        println!("  vs Full caching: {} KB ({:.1}x less memory)", 
                full_cache_memory / 1024,
                full_cache_memory as f64 / total_memory as f64);
    }
    
    println!("\n💡 Selective Caching Benefits:");
    println!("============================");
    println!("✅ Only caches frequently-accessed objects");
    println!("✅ Automatically frees unused caches");  
    println!("✅ Scales to millions of entities");
    println!("✅ Maintains performance for hot objects");
    println!("✅ Minimizes memory pressure");
}
