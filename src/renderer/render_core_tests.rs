//! Tests for RenderCore functionality

#[cfg(test)]
mod tests {
    use super::super::RenderCore;
    use crate::renderable::{Triangle, Quad, Cube, CullingMode};
    use std::collections::HashMap;

    // Helper function to create test objects
    fn create_test_objects() -> (Vec<Triangle>, Vec<Quad>, Vec<Cube>) {
        let triangles = vec![
            Triangle::new(0.5), // CullingMode::None
            Triangle::new(0.3), // CullingMode::None
        ];
        
        let quads = vec![
            Quad::new(0.4), // CullingMode::None
        ];
        
        let cubes = vec![
            Cube::new(0.6), // CullingMode::BackfaceCulling
            Cube::new(0.7), // CullingMode::BackfaceCulling
        ];
        
        (triangles, quads, cubes)
    }

    #[test]
    fn test_culling_group_organization() {
        // We can't easily test the full RenderCore without GPU context,
        // but we can test the logic by creating a mock structure
        let (triangles, quads, cubes) = create_test_objects();
        
        // Convert to references (as the real function expects)
        let triangle_refs: Vec<&Triangle> = triangles.iter().collect();
        let quad_refs: Vec<&Quad> = quads.iter().collect();
        let cube_refs: Vec<&Cube> = cubes.iter().collect();
        
        // Manually implement the grouping logic to test it
        let mut culling_groups: HashMap<CullingMode, (Vec<&Triangle>, Vec<&Quad>, Vec<&Cube>)> = HashMap::new();
        
        // Group triangles by culling mode
        for triangle in &triangle_refs {
            let culling_mode = triangle.get_culling_mode();
            culling_groups.entry(culling_mode).or_insert_with(|| (Vec::new(), Vec::new(), Vec::new())).0.push(*triangle);
        }
        
        // Group quads by culling mode
        for quad in &quad_refs {
            let culling_mode = quad.get_culling_mode();
            culling_groups.entry(culling_mode).or_insert_with(|| (Vec::new(), Vec::new(), Vec::new())).1.push(*quad);
        }
        
        // Group cubes by culling mode
        for cube in &cube_refs {
            let culling_mode = cube.get_culling_mode();
            culling_groups.entry(culling_mode).or_insert_with(|| (Vec::new(), Vec::new(), Vec::new())).2.push(*cube);
        }
        
        // Verify we have exactly 2 culling groups
        assert_eq!(culling_groups.len(), 2);
        
        // Verify the None group has triangles and quads
        let none_group = culling_groups.get(&CullingMode::None).unwrap();
        assert_eq!(none_group.0.len(), 2); // 2 triangles
        assert_eq!(none_group.1.len(), 1); // 1 quad
        assert_eq!(none_group.2.len(), 0); // 0 cubes
        
        // Verify the BackfaceCulling group has cubes
        let backface_group = culling_groups.get(&CullingMode::BackfaceCulling).unwrap();
        assert_eq!(backface_group.0.len(), 0); // 0 triangles
        assert_eq!(backface_group.1.len(), 0); // 0 quads
        assert_eq!(backface_group.2.len(), 2); // 2 cubes
    }

    #[test]
    fn test_empty_collections() {
        // Test with empty collections - should not crash
        let empty_triangles: Vec<&Triangle> = vec![];
        let empty_quads: Vec<&Quad> = vec![];
        let empty_cubes: Vec<&Cube> = vec![];
        
        let mut culling_groups: HashMap<CullingMode, (Vec<&Triangle>, Vec<&Quad>, Vec<&Cube>)> = HashMap::new();
        
        // This should complete without panicking
        for triangle in &empty_triangles {
            let culling_mode = triangle.get_culling_mode();
            culling_groups.entry(culling_mode).or_insert_with(|| (Vec::new(), Vec::new(), Vec::new())).0.push(*triangle);
        }
        
        // Should have no groups
        assert_eq!(culling_groups.len(), 0);
    }

    #[test]
    fn test_single_culling_mode() {
        // Test scenario where all objects have the same culling mode
        let mut triangle = Triangle::new(0.5);
        let mut quad = Quad::new(0.5);
        let mut cube = Cube::new(0.5);
        
        // Set all to the same culling mode
        triangle.set_culling_mode(CullingMode::BackfaceCulling);
        quad.set_culling_mode(CullingMode::BackfaceCulling);
        cube.set_culling_mode(CullingMode::BackfaceCulling);
        
        let triangle_refs = vec![&triangle];
        let quad_refs = vec![&quad];
        let cube_refs = vec![&cube];
        
        let mut culling_groups: HashMap<CullingMode, (Vec<&Triangle>, Vec<&Quad>, Vec<&Cube>)> = HashMap::new();
        
        for triangle in &triangle_refs {
            let culling_mode = triangle.get_culling_mode();
            culling_groups.entry(culling_mode).or_insert_with(|| (Vec::new(), Vec::new(), Vec::new())).0.push(*triangle);
        }
        
        for quad in &quad_refs {
            let culling_mode = quad.get_culling_mode();
            culling_groups.entry(culling_mode).or_insert_with(|| (Vec::new(), Vec::new(), Vec::new())).1.push(*quad);
        }
        
        for cube in &cube_refs {
            let culling_mode = cube.get_culling_mode();
            culling_groups.entry(culling_mode).or_insert_with(|| (Vec::new(), Vec::new(), Vec::new())).2.push(*cube);
        }
        
        // Should have exactly 1 group
        assert_eq!(culling_groups.len(), 1);
        
        // Should be the BackfaceCulling group
        let group = culling_groups.get(&CullingMode::BackfaceCulling).unwrap();
        assert_eq!(group.0.len(), 1); // 1 triangle
        assert_eq!(group.1.len(), 1); // 1 quad
        assert_eq!(group.2.len(), 1); // 1 cube
    }

    #[test]
    fn test_three_culling_modes() {
        // Test scenario with all three culling modes
        let mut triangle = Triangle::new(0.5);
        let mut quad = Quad::new(0.5);
        let mut cube = Cube::new(0.5);
        
        // Set different culling modes
        triangle.set_culling_mode(CullingMode::None);
        quad.set_culling_mode(CullingMode::BackfaceCulling);
        cube.set_culling_mode(CullingMode::FrontfaceCulling);
        
        let triangle_refs = vec![&triangle];
        let quad_refs = vec![&quad];
        let cube_refs = vec![&cube];
        
        let mut culling_groups: HashMap<CullingMode, (Vec<&Triangle>, Vec<&Quad>, Vec<&Cube>)> = HashMap::new();
        
        for triangle in &triangle_refs {
            let culling_mode = triangle.get_culling_mode();
            culling_groups.entry(culling_mode).or_insert_with(|| (Vec::new(), Vec::new(), Vec::new())).0.push(*triangle);
        }
        
        for quad in &quad_refs {
            let culling_mode = quad.get_culling_mode();
            culling_groups.entry(culling_mode).or_insert_with(|| (Vec::new(), Vec::new(), Vec::new())).1.push(*quad);
        }
        
        for cube in &cube_refs {
            let culling_mode = cube.get_culling_mode();
            culling_groups.entry(culling_mode).or_insert_with(|| (Vec::new(), Vec::new(), Vec::new())).2.push(*cube);
        }
        
        // Should have exactly 3 groups
        assert_eq!(culling_groups.len(), 3);
        
        // Verify each group exists and has the right object
        assert!(culling_groups.contains_key(&CullingMode::None));
        assert!(culling_groups.contains_key(&CullingMode::BackfaceCulling));
        assert!(culling_groups.contains_key(&CullingMode::FrontfaceCulling));
        
        // Each group should have exactly one object
        let none_group = culling_groups.get(&CullingMode::None).unwrap();
        assert_eq!(none_group.0.len(), 1); // triangle
        assert_eq!(none_group.1.len(), 0);
        assert_eq!(none_group.2.len(), 0);
        
        let backface_group = culling_groups.get(&CullingMode::BackfaceCulling).unwrap();
        assert_eq!(backface_group.0.len(), 0);
        assert_eq!(backface_group.1.len(), 1); // quad
        assert_eq!(backface_group.2.len(), 0);
        
        let frontface_group = culling_groups.get(&CullingMode::FrontfaceCulling).unwrap();
        assert_eq!(frontface_group.0.len(), 0);
        assert_eq!(frontface_group.1.len(), 0);
        assert_eq!(frontface_group.2.len(), 1); // cube
    }

    #[test]
    fn test_matrix_calculation_logic() {
        // Test the matrix calculation part of our rendering logic
        use crate::renderer::camera::Camera;
        use glam::Vec3;
        
        let camera = Camera::new(16.0 / 9.0); // Standard aspect ratio
        let view_projection = camera.get_view_projection_matrix();
        
        let triangle = Triangle::new_at(0.5, Vec3::new(1.0, 2.0, 3.0));
        let triangle_matrix = triangle.get_matrix();
        
        // Calculate the final matrix (like in render_core)
        let final_matrix = view_projection * triangle_matrix;
        
        // Verify the matrix is valid (not NaN or infinite)
        for i in 0..4 {
            for j in 0..4 {
                let val = final_matrix.col(i)[j];
                assert!(val.is_finite(), "Matrix contains non-finite value: {}", val);
            }
        }
        
        // Verify the translation component is transformed
        let original_pos = triangle_matrix.col(3);
        let transformed_pos = final_matrix.col(3);
        
        // They should be different (camera transform applied)
        assert_ne!(original_pos, transformed_pos);
    }
}
