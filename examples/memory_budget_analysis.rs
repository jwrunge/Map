/// Memory Budget Guidelines for Game Development
/// 
/// This provides realistic memory pressure thresholds for different platforms
/// to help make informed decisions about caching strategies.

use std::fmt;

#[derive(Debug, Clone)]
pub struct MemoryBudget {
    /// Total system RAM available
    pub total_system_ram_gb: f32,
    /// RAM reserved for OS and other processes
    pub os_reserved_gb: f32,
    /// Game's available RAM budget
    pub available_ram_gb: f32,
    /// Recommended limit for transform caches specifically
    pub transform_cache_limit_mb: f32,
    /// Maximum entities before cache becomes problematic
    pub entity_limit_for_full_cache: usize,
}

impl MemoryBudget {
    /// Desktop/Console gaming PC (primary focus, no multitasking)
    pub fn desktop_dedicated() -> Self {
        Self {
            total_system_ram_gb: 16.0,  // Modern gaming PC baseline
            os_reserved_gb: 2.0,         // Windows/Linux OS overhead
            available_ram_gb: 14.0,     // Available for game
            transform_cache_limit_mb: 200.0, // Conservative 1.4% of available RAM
            entity_limit_for_full_cache: 2_500_000, // 200MB / 80 bytes per entity
        }
    }
    
    /// Desktop/Console with multitasking (Discord, browser, etc.)
    pub fn desktop_multitasking() -> Self {
        Self {
            total_system_ram_gb: 16.0,
            os_reserved_gb: 6.0,         // OS + background apps
            available_ram_gb: 10.0,
            transform_cache_limit_mb: 100.0, // 1% of available RAM
            entity_limit_for_full_cache: 1_250_000,
        }
    }
    
    /// High-end console (PS5, Xbox Series X)
    pub fn console_high_end() -> Self {
        Self {
            total_system_ram_gb: 16.0,  // 16GB GDDR6 (shared with GPU)
            os_reserved_gb: 3.0,         // Console OS + system apps
            available_ram_gb: 13.0,     // Available for games
            transform_cache_limit_mb: 150.0, // ~1.2% of available
            entity_limit_for_full_cache: 1_875_000,
        }
    }
    
    /// Mid-tier console (Switch in docked mode)
    pub fn console_mid_tier() -> Self {
        Self {
            total_system_ram_gb: 4.0,   // Nintendo Switch
            os_reserved_gb: 1.0,         // Nintendo OS
            available_ram_gb: 3.0,
            transform_cache_limit_mb: 30.0, // 1% of available RAM
            entity_limit_for_full_cache: 375_000,
        }
    }
    
    /// High-end mobile (iPhone 15 Pro, flagship Android)
    pub fn mobile_high_end() -> Self {
        Self {
            total_system_ram_gb: 8.0,   // iPhone 15 Pro, Samsung S24+
            os_reserved_gb: 3.0,         // iOS/Android + background apps
            available_ram_gb: 5.0,
            transform_cache_limit_mb: 25.0, // 0.5% - mobile is much more conservative
            entity_limit_for_full_cache: 312_500,
        }
    }
    
    /// Mid-tier mobile (iPhone 12, mid-range Android)
    pub fn mobile_mid_tier() -> Self {
        Self {
            total_system_ram_gb: 6.0,   // iPhone 12, Pixel 6a
            os_reserved_gb: 2.5,         // OS + essential apps
            available_ram_gb: 3.5,
            transform_cache_limit_mb: 15.0, // 0.4% of available
            entity_limit_for_full_cache: 187_500,
        }
    }
    
    /// Low-end mobile (budget Android, older iPhones)
    pub fn mobile_low_end() -> Self {
        Self {
            total_system_ram_gb: 3.0,   // Budget Android phones
            os_reserved_gb: 1.5,         // OS overhead is proportionally higher
            available_ram_gb: 1.5,
            transform_cache_limit_mb: 5.0, // 0.33% - very conservative
            entity_limit_for_full_cache: 62_500,
        }
    }
    
    /// Calculate if a given entity count fits within budget
    pub fn fits_entity_count(&self, entities: usize) -> MemoryPressure {
        let cache_usage_mb = (entities * 80) as f32 / 1_048_576.0; // 80 bytes per entity
        let pressure_ratio = cache_usage_mb / self.transform_cache_limit_mb;
        
        match pressure_ratio {
            p if p < 0.25 => MemoryPressure::None,
            p if p < 0.5 => MemoryPressure::Low,
            p if p < 0.75 => MemoryPressure::Moderate,
            p if p < 1.0 => MemoryPressure::High,
            _ => MemoryPressure::Critical,
        }
    }
    
    /// Get recommended entity counts for different pressure levels
    pub fn entity_recommendations(&self) -> EntityRecommendations {
        let bytes_per_entity = 80.0;
        let limit_bytes = self.transform_cache_limit_mb * 1_048_576.0;
        
        EntityRecommendations {
            conservative: (limit_bytes * 0.25 / bytes_per_entity) as usize,
            moderate: (limit_bytes * 0.5 / bytes_per_entity) as usize,
            aggressive: (limit_bytes * 0.75 / bytes_per_entity) as usize,
            maximum: (limit_bytes / bytes_per_entity) as usize,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MemoryPressure {
    None,      // < 25% of budget
    Low,       // 25-50% of budget  
    Moderate,  // 50-75% of budget
    High,      // 75-100% of budget
    Critical,  // > 100% of budget
}

#[derive(Debug, Clone)]
pub struct EntityRecommendations {
    pub conservative: usize, // 25% of budget - always safe
    pub moderate: usize,     // 50% of budget - generally safe
    pub aggressive: usize,   // 75% of budget - requires monitoring
    pub maximum: usize,      // 100% of budget - requires selective caching
}

impl fmt::Display for MemoryBudget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, 
            "RAM: {:.1}GB total, {:.1}GB available | Transform cache limit: {:.0}MB | Max entities: {}",
            self.total_system_ram_gb,
            self.available_ram_gb, 
            self.transform_cache_limit_mb,
            self.entity_limit_for_full_cache
        )
    }
}

impl fmt::Display for MemoryPressure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryPressure::None => write!(f, "🟢 NONE - Full caching recommended"),
            MemoryPressure::Low => write!(f, "🟡 LOW - Full caching acceptable"),
            MemoryPressure::Moderate => write!(f, "🟠 MODERATE - Monitor memory usage"),
            MemoryPressure::High => write!(f, "🔴 HIGH - Consider selective caching"),
            MemoryPressure::Critical => write!(f, "❌ CRITICAL - Selective caching required"),
        }
    }
}

/// Demonstrate memory budgets across different platforms
fn main() {
    println!("🎮 Game Memory Budget Analysis");
    println!("==============================");
    println!();
    
    let platforms = vec![
        ("Desktop (Dedicated Gaming)", MemoryBudget::desktop_dedicated()),
        ("Desktop (Multitasking)", MemoryBudget::desktop_multitasking()),
        ("Console (High-end)", MemoryBudget::console_high_end()),
        ("Console (Mid-tier)", MemoryBudget::console_mid_tier()),
        ("Mobile (High-end)", MemoryBudget::mobile_high_end()),
        ("Mobile (Mid-tier)", MemoryBudget::mobile_mid_tier()),
        ("Mobile (Low-end)", MemoryBudget::mobile_low_end()),
    ];
    
    // Test with different entity counts
    let test_entity_counts = vec![10_000, 50_000, 100_000, 500_000, 1_000_000];
    
    for (platform_name, budget) in &platforms {
        println!("📱 {}", platform_name);
        println!("   {}", budget);
        
        let recommendations = budget.entity_recommendations();
        println!("   Entity Recommendations:");
        println!("     Conservative: {} entities", recommendations.conservative);
        println!("     Moderate:     {} entities", recommendations.moderate);
        println!("     Aggressive:   {} entities", recommendations.aggressive);
        println!("     Maximum:      {} entities", recommendations.maximum);
        
        println!("   Pressure Analysis:");
        for &entity_count in &test_entity_counts {
            let pressure = budget.fits_entity_count(entity_count);
            let cache_mb = (entity_count * 80) as f32 / 1_048_576.0;
            println!("     {} entities ({:.1}MB): {}", 
                    entity_count, cache_mb, pressure);
        }
        println!();
    }
    
    // Practical recommendations
    println!("💡 Practical Guidelines:");
    println!("=======================");
    println!();
    
    println!("🖥️  **Desktop/Console Games (Primary Focus)**");
    println!("   • Transform cache budget: 100-200MB");
    println!("   • Entity limit: 1-2.5 million with full caching");
    println!("   • Strategy: Full caching for most games, selective for massive worlds");
    println!();
    
    println!("📱 **Mobile Games (High-end devices)**");
    println!("   • Transform cache budget: 20-25MB");
    println!("   • Entity limit: 250-300K with full caching");
    println!("   • Strategy: Full caching for small games, selective for larger ones");
    println!();
    
    println!("📱 **Mobile Games (Mid-tier devices)**");
    println!("   • Transform cache budget: 10-15MB");
    println!("   • Entity limit: 125-200K with full caching");
    println!("   • Strategy: Selective caching recommended above 100K entities");
    println!();
    
    println!("📱 **Mobile Games (Low-end devices)**");
    println!("   • Transform cache budget: 3-5MB");
    println!("   • Entity limit: 40-60K with full caching");
    println!("   • Strategy: Very conservative, disable caching if needed");
    println!();
    
    println!("🎯 **Your Light & Performant Project**");
    println!("   • Current approach: ✅ Excellent for all platforms");
    println!("   • Full caching safe up to:");
    println!("     - Desktop: 1M+ entities");
    println!("     - Mobile high-end: 250K entities");
    println!("     - Mobile mid-tier: 150K entities");
    println!("     - Mobile low-end: 50K entities");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_pressure_calculation() {
        let budget = MemoryBudget::mobile_mid_tier();
        
        // 50K entities should be low pressure
        assert_eq!(budget.fits_entity_count(50_000), MemoryPressure::Low);
        
        // 200K entities should be critical pressure
        assert_eq!(budget.fits_entity_count(200_000), MemoryPressure::Critical);
    }
    
    #[test]
    fn test_entity_recommendations() {
        let budget = MemoryBudget::desktop_dedicated();
        let recs = budget.entity_recommendations();
        
        // Conservative should be much less than maximum
        assert!(recs.conservative < recs.maximum);
        assert!(recs.moderate > recs.conservative);
        assert!(recs.aggressive > recs.moderate);
    }
    
    #[test]
    fn test_platform_scaling() {
        let desktop = MemoryBudget::desktop_dedicated();
        let mobile = MemoryBudget::mobile_low_end();
        
        // Desktop should support many more entities than mobile
        assert!(desktop.entity_limit_for_full_cache > mobile.entity_limit_for_full_cache * 10);
    }
}
