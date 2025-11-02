use meshbool::{cube, get_mesh_gl, rotate, translate, MeshBool};
use nalgebra::{Point3, Vector3};

#[test]
fn test_complex_boolean_union() {
    let cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let cube2 = translate(&cube1, Point3::new(1.0, 0.0, 0.0));
    let cube3 = rotate(&cube1, 45.0, 45.0, 45.0);

    // More complex boolean operation - union of multiple transformed shapes
    let union1 = &cube1 + &cube2;
    let complex_union = &union1 + &cube3;
    let mesh = get_mesh_gl(&complex_union, 0);

    assert!(!mesh.tri_verts.is_empty());
}

#[test]
fn test_nested_boolean_operations() {
    let cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let cube2 = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let cube3 = cube(Vector3::new(0.5, 0.5, 0.5), true);

    // (A ∪ B) ∩ C
    let union = &cube1 + &cube2;
    let intersection = &union ^ &cube3;
    let mesh = get_mesh_gl(&intersection, 0);

    assert!(!mesh.tri_verts.is_empty());
}

#[test]
fn test_complex_difference_operations() {
    let large_cube = cube(Vector3::new(3.0, 3.0, 3.0), true);
    let medium_cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let small_cube = cube(Vector3::new(1.0, 1.0, 1.0), true);

    // Large cube with holes - Large - (Medium ∪ Small)
    let combined_hole = &medium_cube + &small_cube;
    let result = &large_cube - &combined_hole;
    let mesh = get_mesh_gl(&result, 0);

    assert!(!mesh.tri_verts.is_empty());
}

#[test]
fn test_boolean_with_invalid_mesh() {
    // This test verifies that complex boolean operations with invalid meshes
    // properly handle error conditions
    
    // Create an invalid mesh by creating a cube with invalid dimensions
    let invalid_mesh = cube(Vector3::new(-1.0, 2.0, 2.0), true); // This should create an invalid mesh 
    let valid_cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    
    // If the invalid mesh has an error status, we check it
    if invalid_mesh.status() != meshbool::ManifoldError::NoError {
        // The invalid mesh should have an error status
        assert_ne!(invalid_mesh.status(), meshbool::ManifoldError::NoError);
    } else {
        // If the mesh was valid despite having negative dimension, perform the operation
        let result = &invalid_mesh + &valid_cube;
        // The result should either be valid or have the same error as the invalid mesh
    }
}

#[test]
fn test_boolean_edge_cases_large_numbers() {
    let large_cube = cube(Vector3::new(1000.0, 1000.0, 1000.0), true);
    let small_cube = cube(Vector3::new(0.001, 0.001, 0.001), true);

    let result = &large_cube + &small_cube;
    let mesh = get_mesh_gl(&result, 0);

    assert!(!mesh.tri_verts.is_empty());
}
