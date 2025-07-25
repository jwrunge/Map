//! Tests for Camera and projection system
//! 
//! Validates view/projection matrix calculations and camera behavior

use crate::renderer::camera::{Camera, ProjectionMode};
use glam::{Mat4, Vec3};

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-6;

    fn matrices_approximately_equal(a: &Mat4, b: &Mat4) -> bool {
        for i in 0..16 {
            if (a.to_cols_array()[i] - b.to_cols_array()[i]).abs() > EPSILON {
                return false;
            }
        }
        true
    }

    #[test]
    fn test_camera_creation() {
        let camera = Camera::new(16.0 / 9.0);
        
        assert_eq!(camera.aspect_ratio, 16.0 / 9.0);
        assert_eq!(camera.position, Vec3::new(0.0, 0.0, 3.0));
        assert_eq!(camera.target, Vec3::ZERO);
        assert_eq!(camera.up, Vec3::Y);
        assert_eq!(camera.projection_mode, ProjectionMode::Perspective);
        
        // Verify matrices are not identity (they should be updated)
        assert!(!matrices_approximately_equal(&camera.view_matrix, &Mat4::IDENTITY));
        assert!(!matrices_approximately_equal(&camera.projection_matrix, &Mat4::IDENTITY));
    }

    #[test]
    fn test_orthographic_2d_camera() {
        let camera = Camera::orthographic_2d(4.0 / 3.0);
        
        assert_eq!(camera.aspect_ratio, 4.0 / 3.0);
        assert_eq!(camera.position, Vec3::new(0.0, 0.0, 1.0));
        assert_eq!(camera.target, Vec3::ZERO);
        assert_eq!(camera.up, Vec3::Y);
        assert_eq!(camera.projection_mode, ProjectionMode::Orthographic);
    }

    #[test]
    fn test_projection_mode_toggle() {
        let mut camera = Camera::new(1.0);
        
        // Start with perspective
        assert_eq!(camera.projection_mode, ProjectionMode::Perspective);
        
        // Switch to orthographic
        camera.set_projection_mode(ProjectionMode::Orthographic);
        assert_eq!(camera.projection_mode, ProjectionMode::Orthographic);
        
        // Switch back to perspective
        camera.set_projection_mode(ProjectionMode::Perspective);
        assert_eq!(camera.projection_mode, ProjectionMode::Perspective);
    }

    #[test]
    fn test_camera_movement() {
        let mut camera = Camera::new(1.0);
        let initial_position = camera.position;
        let initial_view = camera.view_matrix;
        
        // Move camera
        let new_position = Vec3::new(1.0, 2.0, 3.0);
        camera.set_position(new_position);
        
        assert_eq!(camera.position, new_position);
        // View matrix should have changed
        assert!(!matrices_approximately_equal(&camera.view_matrix, &initial_view));
    }

    #[test]
    fn test_camera_target() {
        let mut camera = Camera::new(1.0);
        let initial_target = camera.target;
        let initial_view = camera.view_matrix;
        
        // Change target
        let new_target = Vec3::new(5.0, -2.0, 1.0);
        camera.set_target(new_target);
        
        assert_eq!(camera.target, new_target);
        // View matrix should have changed
        assert!(!matrices_approximately_equal(&camera.view_matrix, &initial_view));
    }

    #[test]
    fn test_aspect_ratio_update() {
        let mut camera = Camera::new(1.0);
        let initial_projection = camera.projection_matrix;
        
        // Change aspect ratio
        camera.set_aspect_ratio(2.0);
        assert_eq!(camera.aspect_ratio, 2.0);
        
        // Projection matrix should have changed
        assert!(!matrices_approximately_equal(&camera.projection_matrix, &initial_projection));
    }

    #[test]
    fn test_different_aspect_ratios() {
        let aspect_ratios = [0.5, 1.0, 1.33, 1.78, 2.0, 3.0];
        
        for &aspect_ratio in &aspect_ratios {
            let camera = Camera::new(aspect_ratio);
            assert_eq!(camera.aspect_ratio, aspect_ratio);
            
            // Verify projection matrix is reasonable (not NaN or infinite)
            let proj_array = camera.projection_matrix.to_cols_array();
            for &value in &proj_array {
                assert!(value.is_finite(), "Projection matrix contains non-finite value");
            }
        }
    }

    #[test]
    fn test_projection_mode_matrix_differences() {
        let mut camera = Camera::new(1.0);
        
        // Get perspective projection matrix
        camera.set_projection_mode(ProjectionMode::Perspective);
        let perspective_matrix = camera.projection_matrix;
        
        // Get orthographic projection matrix
        camera.set_projection_mode(ProjectionMode::Orthographic);
        let orthographic_matrix = camera.projection_matrix;
        
        // They should be different
        assert!(!matrices_approximately_equal(&perspective_matrix, &orthographic_matrix));
    }

    #[test]
    fn test_view_matrix_calculation() {
        let mut camera = Camera::new(1.0);
        
        // Test looking at origin from positive Z
        camera.set_position(Vec3::new(0.0, 0.0, 5.0));
        camera.set_target(Vec3::ZERO);
        
        // View matrix should transform world coordinates appropriately
        let view_matrix = camera.view_matrix;
        
        // Origin should map to (0, 0, -5) in view space
        let origin_view = view_matrix * Vec3::ZERO.extend(1.0);
        assert!((origin_view.z + 5.0).abs() < EPSILON, "Origin Z should be -5 in view space");
    }

    #[test]
    fn test_camera_position_extremes() {
        let mut camera = Camera::new(1.0);
        
        // Test very large positions
        camera.set_position(Vec3::new(1000.0, -1000.0, 500.0));
        assert!(camera.view_matrix.to_cols_array().iter().all(|&x| x.is_finite()));
        
        // Test very small positions
        camera.set_position(Vec3::new(0.001, -0.001, 0.1));
        assert!(camera.view_matrix.to_cols_array().iter().all(|&x| x.is_finite()));
        
        // Test zero position
        camera.set_position(Vec3::ZERO);
        assert!(camera.view_matrix.to_cols_array().iter().all(|&x| x.is_finite()));
    }

    #[test]
    fn test_camera_up_vector() {
        let mut camera = Camera::new(1.0);
        let initial_up = camera.up;
        
        // Change up vector
        let new_up = Vec3::new(0.0, 0.0, 1.0); // Z up instead of Y up
        camera.set_up(new_up);
        
        assert_eq!(camera.up, new_up);
        assert_ne!(camera.up, initial_up);
    }

    #[test]
    fn test_matrix_consistency() {
        let camera = Camera::new(1.777);
        
        // Verify that matrices are internally consistent
        // View matrix should be invertible
        let view_det = camera.view_matrix.determinant();
        assert!(view_det.abs() > EPSILON, "View matrix should be invertible");
        
        // Projection matrix should be reasonable for perspective
        if camera.projection_mode == ProjectionMode::Perspective {
            let proj_det = camera.projection_matrix.determinant();
            assert!(proj_det.abs() > EPSILON, "Projection matrix should be invertible");
        }
    }

    #[test]
    fn test_camera_look_at_behavior() {
        let mut camera = Camera::new(1.0);
        
        // Position camera and look at a specific point
        camera.set_position(Vec3::new(0.0, 0.0, 5.0));
        camera.set_target(Vec3::new(1.0, 1.0, 0.0));
        
        // The forward vector should point towards the target
        let view_matrix = camera.view_matrix;
        let forward = Vec3::new(-view_matrix.z_axis.x, -view_matrix.z_axis.y, -view_matrix.z_axis.z);
        let expected_forward = (camera.target - camera.position).normalize();
        
        // Check if forward vectors are approximately equal
        let dot_product = forward.dot(expected_forward);
        assert!(dot_product > 0.99, "Camera should be looking towards target");
    }

    #[test]
    fn test_edge_case_same_position_and_target() {
        let mut camera = Camera::new(1.0);
        
        // Set position and target to the same point
        let same_point = Vec3::new(1.0, 2.0, 3.0);
        camera.set_position(same_point);
        camera.set_target(same_point);
        
        // View matrix should still be valid (no NaN/infinite values)
        let view_array = camera.view_matrix.to_cols_array();
        for &value in &view_array {
            assert!(value.is_finite(), "View matrix should remain finite even when position equals target");
        }
    }

    #[test]
    fn test_projection_matrix_properties() {
        let camera = Camera::new(16.0 / 9.0);
        
        match camera.projection_mode {
            ProjectionMode::Perspective => {
                // For perspective projection, far objects should be smaller
                // The (3,2) element should be negative for standard perspective
                let proj = camera.projection_matrix;
                assert!(proj.w_axis.z < 0.0, "Perspective projection should have negative w_axis.z");
            }
            ProjectionMode::Orthographic => {
                // For orthographic projection, no perspective division
                let proj = camera.projection_matrix;
                assert_eq!(proj.w_axis.w, 1.0, "Orthographic projection should have w_axis.w = 1");
            }
        }
    }
}
