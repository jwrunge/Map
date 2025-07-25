//! Tests for renderable objects
//!
//! These tests validate the behavior of Triangle, Quad, and Cube objects,
//! particularly their culling modes and transformations.

#[cfg(test)]
mod tests {
    use super::super::{Triangle, Quad, Cube, Renderable, CullingMode};
    use glam::{Vec3, Mat4};

    #[test]
    fn test_triangle_default_culling_mode() {
        let triangle = Triangle::new(0.5);
        // Triangles should default to no culling (good for 2D)
        assert_eq!(triangle.get_culling_mode(), CullingMode::None);
    }

    #[test]
    fn test_quad_default_culling_mode() {
        let quad = Quad::new(0.5);
        // Quads should default to no culling (good for 2D)
        assert_eq!(quad.get_culling_mode(), CullingMode::None);
    }

    #[test]
    fn test_cube_default_culling_mode() {
        let cube = Cube::new(0.5);
        // Cubes should default to backface culling (good for 3D)
        assert_eq!(cube.get_culling_mode(), CullingMode::BackfaceCulling);
    }

    #[test]
    fn test_triangle_culling_mode_modification() {
        let mut triangle = Triangle::new(0.5);
        
        // Should start with None
        assert_eq!(triangle.get_culling_mode(), CullingMode::None);
        
        // Should be able to change
        triangle.set_culling_mode(CullingMode::BackfaceCulling);
        assert_eq!(triangle.get_culling_mode(), CullingMode::BackfaceCulling);
        
        // Should be able to change back
        triangle.set_culling_mode(CullingMode::None);
        assert_eq!(triangle.get_culling_mode(), CullingMode::None);
    }

    #[test]
    fn test_cube_culling_mode_modification() {
        let mut cube = Cube::new(0.5);
        
        // Should start with BackfaceCulling
        assert_eq!(cube.get_culling_mode(), CullingMode::BackfaceCulling);
        
        // Should be able to change to None (useful for debug)
        cube.set_culling_mode(CullingMode::None);
        assert_eq!(cube.get_culling_mode(), CullingMode::None);
        
        // Should be able to change to FrontfaceCulling
        cube.set_culling_mode(CullingMode::FrontfaceCulling);
        assert_eq!(cube.get_culling_mode(), CullingMode::FrontfaceCulling);
    }

    #[test]
    fn test_object_transformations() {
        let mut triangle = Triangle::new(0.5);
        
        // Initial position should be at origin
        let initial_matrix = triangle.get_matrix();
        let initial_translation = initial_matrix.col(3).truncate();
        assert_eq!(initial_translation, Vec3::ZERO);
        
        // Test translation
        triangle.set_position(Vec3::new(1.0, 2.0, 3.0));
        let translated_matrix = triangle.get_matrix();
        let translation = translated_matrix.col(3).truncate();
        assert_eq!(translation, Vec3::new(1.0, 2.0, 3.0));
        
        // Test that scale is preserved
        let scale_factor = translated_matrix.col(0).length();
        assert!((scale_factor - 0.5).abs() < 0.001); // Should still be 0.5
    }

    #[test]
    fn test_vertex_provider_interface() {
        let triangle = Triangle::new(0.3);
        let quad = Quad::new(0.4);
        let cube = Cube::new(0.5);
        
        // All objects should provide vertices
        let triangle_vertices = triangle.get_vertices();
        let quad_vertices = quad.get_vertices();
        let cube_vertices = cube.get_vertices();
        
        // Triangle should have 3 vertices
        assert_eq!(triangle_vertices.len(), 3);
        
        // Quad should have 6 vertices (2 triangles)
        assert_eq!(quad_vertices.len(), 6);
        
        // Cube should have 36 vertices (12 triangles * 3 vertices)
        assert_eq!(cube_vertices.len(), 36);
    }

    #[test]
    fn test_object_creation_with_position() {
        let pos = Vec3::new(1.0, -2.0, 3.5);
        
        let triangle = Triangle::new_at(0.5, pos);
        let quad = Quad::new_at(0.5, pos);
        let cube = Cube::new_at(0.5, pos);
        
        // All should be at the specified position
        assert_eq!(triangle.get_matrix().col(3).truncate(), pos);
        assert_eq!(quad.get_matrix().col(3).truncate(), pos);
        assert_eq!(cube.get_matrix().col(3).truncate(), pos);
    }

    #[test]
    fn test_object_scaling() {
        let small_triangle = Triangle::new(0.1);
        let large_triangle = Triangle::new(1.0);
        
        let small_vertices = small_triangle.get_vertices();
        let large_vertices = large_triangle.get_vertices();
        
        // First vertex should be scaled appropriately
        let small_pos = Vec3::new(small_vertices[0].position[0], small_vertices[0].position[1], small_vertices[0].position[2]);
        let large_pos = Vec3::new(large_vertices[0].position[0], large_vertices[0].position[1], large_vertices[0].position[2]);
        
        // Large should be 10x bigger than small
        let scale_ratio = large_pos.length() / small_pos.length();
        assert!((scale_ratio - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_mixed_culling_modes_scenario() {
        let triangle = Triangle::new(0.5); // CullingMode::None
        let quad = Quad::new(0.5);         // CullingMode::None  
        let cube = Cube::new(0.5);         // CullingMode::BackfaceCulling
        
        let triangle_mode = triangle.get_culling_mode();
        let quad_mode = quad.get_culling_mode();
        let cube_mode = cube.get_culling_mode();
        
        assert_eq!(triangle_mode, quad_mode); // Both None
        assert_ne!(triangle_mode, cube_mode); // Triangle None != Cube BackfaceCulling
        assert_ne!(quad_mode, cube_mode);     // Quad None != Cube BackfaceCulling
        
        // This validates our render grouping logic will work correctly
        assert_eq!(triangle_mode, CullingMode::None);
        assert_eq!(quad_mode, CullingMode::None);
        assert_eq!(cube_mode, CullingMode::BackfaceCulling);
    }
}
