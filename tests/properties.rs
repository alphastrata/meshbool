use meshbool::{cube, get_mesh_gl, translate};

#[test]
fn test_basic_properties() {
    use nalgebra::Vector3;
    
    let cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let mesh = get_mesh_gl(&cube, 0);
    
    // Verify that the mesh has the expected properties
    assert!(mesh.num_prop >= 3); // x, y, z coordinates
    assert!(!mesh.tri_verts.is_empty());
    assert!(!mesh.vert_properties.is_empty());
    
    // Each vertex should have the expected number of properties
    let num_vertices = mesh.vert_properties.len() / mesh.num_prop as usize;
    assert!(num_vertices > 0);
    
    // Verify that the number of property values is consistent
    assert_eq!(mesh.vert_properties.len(), (num_vertices * mesh.num_prop as usize));
}

#[test]
fn test_property_consistency_after_transform() {
    use nalgebra::{Vector3, Point3};
    
    let original_cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let original_mesh = get_mesh_gl(&original_cube, 0);
    
    // Transform the cube
    let translated_cube = translate(&original_cube, Point3::new(5.0, 0.0, 0.0));
    let transformed_mesh = get_mesh_gl(&translated_cube, 0);
    
    // Properties should be maintained after transform
    assert_eq!(original_mesh.num_prop, transformed_mesh.num_prop);
    assert_eq!(original_mesh.tri_verts.len(), transformed_mesh.tri_verts.len());
    
    // Both should have the same number of vertices
    let orig_num_verts = original_mesh.vert_properties.len() / original_mesh.num_prop as usize;
    let trans_num_verts = transformed_mesh.vert_properties.len() / transformed_mesh.num_prop as usize;
    assert_eq!(orig_num_verts, trans_num_verts);
}

#[test]
fn test_property_values() {
    use nalgebra::Vector3;
    
    let small_cube = cube(Vector3::new(0.5, 0.5, 0.5), true);
    let mesh = get_mesh_gl(&small_cube, 0);
    
    // Check that vertex properties are within expected bounds for a centered cube of size 0.5
    for i in 0..(mesh.vert_properties.len() / mesh.num_prop as usize) {
        let x = mesh.vert_properties[i * mesh.num_prop as usize + 0];
        let y = mesh.vert_properties[i * mesh.num_prop as usize + 1];
        let z = mesh.vert_properties[i * mesh.num_prop as usize + 2];
        
        // For a 0.5x0.5x0.5 centered cube, coordinates should be within [-0.25, 0.25]
        assert!(x >= -0.25 && x <= 0.25);
        assert!(y >= -0.25 && y <= 0.25);
        assert!(z >= -0.25 && z <= 0.25);
    }
}

#[test]
fn test_property_merge_handling() {
    use nalgebra::Vector3;
    
    let cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let mesh1 = get_mesh_gl(&cube1, 0);
    
    // Check that merge vectors are properly handled
    // This tests the relationship between merge_from_vert and merge_to_vert
    if !mesh1.merge_from_vert.is_empty() {
        assert_eq!(mesh1.merge_from_vert.len(), mesh1.merge_to_vert.len());
        
        // Check that merge indices are within valid range
        let num_vertices = mesh1.vert_properties.len() / mesh1.num_prop as usize;
        for &from_idx in &mesh1.merge_from_vert {
            assert!((from_idx as usize) < num_vertices);
        }
        for &to_idx in &mesh1.merge_to_vert {
            assert!((to_idx as usize) < num_vertices);
        }
    }
}

#[test]
fn test_property_runs() {
    use nalgebra::Vector3;
    
    let cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let mesh = get_mesh_gl(&cube, 0);
    
    // Check run structure integrity
    assert!(!mesh.run_index.is_empty());
    assert_eq!(mesh.run_original_id.len(), mesh.run_index.len() - 1);
    
    // Each run should have valid index ranges
    for i in 0..(mesh.run_index.len() - 1) {
        let start = mesh.run_index[i];
        let end = mesh.run_index[i + 1];
        
        assert!(end >= start);
        assert!(end % 3 == 0);  // Triangle indices should be multiples of 3
        assert!(start % 3 == 0);  // Triangle indices should be multiples of 3
    }
    
    // Check that face IDs are consistent with triangles
    if !mesh.face_id.is_empty() {
        assert_eq!(mesh.face_id.len(), mesh.tri_verts.len() / 3);  // One face ID per triangle
    }
}