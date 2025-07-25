//! Tests for Scene management system
//! 
//! Validates entity creation, management, and rendering pipeline

use crate::scene::{Scene, EntityId};
use crate::renderable::{Triangle, Quad, Cube};
use glam::Vec3;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_creation() {
        let scene = Scene::new();
        assert_eq!(scene.triangle_count(), 0);
        assert_eq!(scene.quad_count(), 0);
        assert_eq!(scene.cube_count(), 0);
    }

    #[test]
    fn test_scene_default() {
        let scene = Scene::default();
        assert_eq!(scene.triangle_count(), 0);
        assert_eq!(scene.quad_count(), 0);
        assert_eq!(scene.cube_count(), 0);
    }

    #[test]
    fn test_add_triangle() {
        let mut scene = Scene::new();
        let triangle = Triangle::new();
        let id = scene.add_triangle(triangle);
        
        assert_eq!(id, 0);
        assert_eq!(scene.triangle_count(), 1);
        assert_eq!(scene.quad_count(), 0);
        assert_eq!(scene.cube_count(), 0);
    }

    #[test]
    fn test_add_quad() {
        let mut scene = Scene::new();
        let quad = Quad::new();
        let id = scene.add_quad(quad);
        
        assert_eq!(id, 0);
        assert_eq!(scene.triangle_count(), 0);
        assert_eq!(scene.quad_count(), 1);
        assert_eq!(scene.cube_count(), 0);
    }

    #[test]
    fn test_add_cube() {
        let mut scene = Scene::new();
        let cube = Cube::new();
        let id = scene.add_cube(cube);
        
        assert_eq!(id, 0);
        assert_eq!(scene.triangle_count(), 0);
        assert_eq!(scene.quad_count(), 0);
        assert_eq!(scene.cube_count(), 1);
    }

    #[test]
    fn test_entity_id_sequencing() {
        let mut scene = Scene::new();
        
        let triangle_id = scene.add_triangle(Triangle::new());
        let quad_id = scene.add_quad(Quad::new());
        let cube_id = scene.add_cube(Cube::new());
        
        assert_eq!(triangle_id, 0);
        assert_eq!(quad_id, 1);
        assert_eq!(cube_id, 2);
    }

    #[test]
    fn test_remove_triangle() {
        let mut scene = Scene::new();
        let triangle = Triangle::new();
        let id = scene.add_triangle(triangle);
        
        assert_eq!(scene.triangle_count(), 1);
        
        let removed = scene.remove_triangle(id);
        assert!(removed.is_some());
        assert_eq!(scene.triangle_count(), 0);
        
        // Try to remove again - should return None
        let removed_again = scene.remove_triangle(id);
        assert!(removed_again.is_none());
    }

    #[test]
    fn test_get_triangle_mut() {
        let mut scene = Scene::new();
        let id = scene.add_triangle(Triangle::new());
        
        {
            let triangle_ref = scene.get_triangle_mut(id);
            assert!(triangle_ref.is_some());
            
            // Modify the triangle through the mutable reference
            if let Some(triangle) = triangle_ref {
                triangle.transform_set_position(Vec3::new(1.0, 2.0, 3.0));
            }
        }
        
        // Verify the triangle was modified
        if let Some(triangle) = scene.get_triangle_mut(id) {
            let position = triangle.transform_get_position();
            assert_eq!(position, Vec3::new(1.0, 2.0, 3.0));
        }
        
        // Test with invalid ID
        let invalid_ref = scene.get_triangle_mut(999);
        assert!(invalid_ref.is_none());
    }

    #[test]
    fn test_create_triangle_methods() {
        let mut scene = Scene::new();
        
        // Test basic triangle creation
        let id1 = scene.create_triangle(2.0);
        assert_eq!(scene.triangle_count(), 1);
        
        // Test triangle at specific position
        let position = Vec3::new(5.0, 10.0, 15.0);
        let id2 = scene.create_triangle_at(1.5, position);
        assert_eq!(scene.triangle_count(), 2);
        
        if let Some(triangle) = scene.get_triangle_mut(id2) {
            assert_eq!(triangle.transform_get_position(), position);
        }
        
        // Test triangle with custom transform
        let rotation = Vec3::new(45.0, 90.0, 180.0);
        let id3 = scene.create_triangle_with_transform(1.0, position, rotation);
        assert_eq!(scene.triangle_count(), 3);
        
        if let Some(triangle) = scene.get_triangle_mut(id3) {
            assert_eq!(triangle.transform_get_position(), position);
        }
    }

    #[test]
    fn test_create_quad_methods() {
        let mut scene = Scene::new();
        
        // Test basic quad creation
        let id1 = scene.create_quad(2.0);
        assert_eq!(scene.quad_count(), 1);
        
        // Test quad at specific position
        let position = Vec3::new(5.0, 10.0, 15.0);
        let id2 = scene.create_quad_at(1.5, position);
        assert_eq!(scene.quad_count(), 2);
        
        // Test quad with custom transform
        let rotation = Vec3::new(45.0, 90.0, 180.0);
        let id3 = scene.create_quad_with_transform(1.0, position, rotation);
        assert_eq!(scene.quad_count(), 3);
    }

    #[test]
    fn test_create_cube_methods() {
        let mut scene = Scene::new();
        
        // Test basic cube creation
        let id1 = scene.create_cube(2.0);
        assert_eq!(scene.cube_count(), 1);
        
        // Test cube at specific position
        let position = Vec3::new(5.0, 10.0, 15.0);
        let id2 = scene.create_cube_at(1.5, position);
        assert_eq!(scene.cube_count(), 2);
        
        // Test cube with custom transform
        let rotation = Vec3::new(45.0, 90.0, 180.0);
        let id3 = scene.create_cube_with_transform(1.0, position, rotation);
        assert_eq!(scene.cube_count(), 3);
    }

    #[test]
    fn test_get_all_renderables() {
        let mut scene = Scene::new();
        
        scene.add_triangle(Triangle::new());
        scene.add_triangle(Triangle::new());
        scene.add_quad(Quad::new());
        scene.add_cube(Cube::new());
        scene.add_cube(Cube::new());
        scene.add_cube(Cube::new());
        
        let (triangles, quads, cubes) = scene.get_all_renderables();
        
        assert_eq!(triangles.len(), 2);
        assert_eq!(quads.len(), 1);
        assert_eq!(cubes.len(), 3);
    }

    #[test]
    fn test_update_all_entities() {
        let mut scene = Scene::new();
        
        scene.add_triangle(Triangle::new());
        scene.add_quad(Quad::new());
        scene.add_cube(Cube::new());
        
        // This should not panic - validates that update() can be called
        scene.update(0.016); // 60 FPS delta time
    }

    #[test]
    fn test_render_batch_callbacks() {
        let mut scene = Scene::new();
        
        scene.add_triangle(Triangle::new());
        scene.add_triangle(Triangle::new());
        scene.add_quad(Quad::new());
        scene.add_cube(Cube::new());
        
        // Test triangle batch rendering
        let mut triangle_count = 0;
        let result = scene.render_triangles_batch(|triangles| {
            triangle_count = triangles.len();
            Ok(())
        });
        assert!(result.is_ok());
        assert_eq!(triangle_count, 2);
        
        // Test quad batch rendering
        let mut quad_count = 0;
        let result = scene.render_quads_batch(|quads| {
            quad_count = quads.len();
            Ok(())
        });
        assert!(result.is_ok());
        assert_eq!(quad_count, 1);
        
        // Test cube batch rendering
        let mut cube_count = 0;
        let result = scene.render_cubes_batch(|cubes| {
            cube_count = cubes.len();
            Ok(())
        });
        assert!(result.is_ok());
        assert_eq!(cube_count, 1);
    }

    #[test]
    fn test_render_triangles_batch_mut() {
        let mut scene = Scene::new();
        
        scene.add_triangle(Triangle::new());
        scene.add_triangle(Triangle::new());
        
        let mut processed_count = 0;
        let result = scene.render_triangles_batch_mut(|triangles| {
            processed_count = triangles.len();
            // Simulate updating dirty flags or other mutable operations
            for triangle in triangles.iter_mut() {
                triangle.transform_set_position(Vec3::new(1.0, 0.0, 0.0));
            }
            Ok(())
        });
        
        assert!(result.is_ok());
        assert_eq!(processed_count, 2);
        
        // Verify triangles were modified
        let (triangles, _, _) = scene.get_all_renderables();
        for triangle in triangles {
            assert_eq!(triangle.transform_get_position(), Vec3::new(1.0, 0.0, 0.0));
        }
    }

    #[test]
    fn test_render_all_callback() {
        let mut scene = Scene::new();
        
        scene.add_triangle(Triangle::new());
        scene.add_quad(Quad::new());
        scene.add_cube(Cube::new());
        
        let mut total_objects = 0;
        
        let result = scene.render_all(|objects: &[&Triangle]| {
            total_objects += objects.len();
            Ok(())
        });
        
        assert!(result.is_ok());
        assert_eq!(total_objects, 3); // 1 triangle + 1 quad + 1 cube
    }

    #[test]
    fn test_error_propagation_in_render_batch() {
        let mut scene = Scene::new();
        scene.add_triangle(Triangle::new());
        
        // Test error propagation from render callback
        let result = scene.render_triangles_batch(|_| {
            Err(wgpu::SurfaceError::Timeout)
        });
        
        assert!(result.is_err());
        match result {
            Err(wgpu::SurfaceError::Timeout) => {},
            _ => panic!("Expected SurfaceError::Timeout"),
        }
    }

    #[test]
    fn test_mixed_entity_operations() {
        let mut scene = Scene::new();
        
        // Add various entities
        let t1 = scene.add_triangle(Triangle::new());
        let q1 = scene.add_quad(Quad::new());
        let c1 = scene.add_cube(Cube::new());
        let t2 = scene.add_triangle(Triangle::new());
        
        assert_eq!(scene.triangle_count(), 2);
        assert_eq!(scene.quad_count(), 1);
        assert_eq!(scene.cube_count(), 1);
        
        // Remove one triangle
        scene.remove_triangle(t1);
        assert_eq!(scene.triangle_count(), 1);
        
        // Add more entities
        scene.add_quad(Quad::new());
        scene.add_cube(Cube::new());
        
        assert_eq!(scene.triangle_count(), 1);
        assert_eq!(scene.quad_count(), 2);
        assert_eq!(scene.cube_count(), 2);
        
        // Verify final state
        let (triangles, quads, cubes) = scene.get_all_renderables();
        assert_eq!(triangles.len(), 1);
        assert_eq!(quads.len(), 2);
        assert_eq!(cubes.len(), 2);
    }

    #[test]
    fn test_large_scene_performance() {
        let mut scene = Scene::new();
        
        // Add a reasonable number of entities
        for i in 0..100 {
            scene.add_triangle(Triangle::new());
            scene.add_quad(Quad::new());
            scene.add_cube(Cube::new());
        }
        
        assert_eq!(scene.triangle_count(), 100);
        assert_eq!(scene.quad_count(), 100);
        assert_eq!(scene.cube_count(), 100);
        
        // Test that get_all_renderables still works efficiently
        let (triangles, quads, cubes) = scene.get_all_renderables();
        assert_eq!(triangles.len(), 100);
        assert_eq!(quads.len(), 100);
        assert_eq!(cubes.len(), 100);
        
        // Test update performance
        scene.update(0.016);
    }
}
