use meshbool::{cube, get_mesh_gl};

#[test]
fn test_basic_boolean_operations() {
    use nalgebra::Vector3;

    // Test basic cube boolean operations - similar to the existing test
    let cube1 = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let cube2 = cube(Vector3::new(1.0, 1.0, 1.0), false);

    // Test union
    let union = &cube1 + &cube2;
    let union_mesh = get_mesh_gl(&union, 0);
    assert!(!union_mesh.tri_verts.is_empty());

    // Test difference
    let difference = &cube1 - &cube2;
    let difference_mesh = get_mesh_gl(&difference, 0);
    assert!(!difference_mesh.tri_verts.is_empty());

    // Test intersection
    let intersection = &cube1 ^ &cube2;
    let intersection_mesh = get_mesh_gl(&intersection, 0);
    assert!(!intersection_mesh.tri_verts.is_empty());
}

#[test]
fn test_cube_union() {
    use nalgebra::Vector3;

    let cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let cube2 = cube(Vector3::new(2.0, 2.0, 2.0), true);

    // Second cube offset to partially overlap with first
    let cube2_transformed = meshbool::translate(&cube2, nalgebra::Point3::new(1.0, 0.0, 0.0));

    let union = &cube1 + &cube2_transformed;
    let mesh = get_mesh_gl(&union, 0);

    // Union should have triangles
    assert!(!mesh.tri_verts.is_empty());
    // Union should be a valid manifold
    assert!(mesh.tri_verts.len() % 3 == 0); // Triangles should have 3 vertices each
}

#[test]
fn test_cube_difference() {
    use nalgebra::Vector3;

    let cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let cube2 = cube(Vector3::new(1.0, 1.0, 1.0), true);

    // Position cube2 inside cube1 to create a hollow shape
    let difference = &cube1 - &cube2;
    let mesh = get_mesh_gl(&difference, 0);

    assert!(!mesh.tri_verts.is_empty());
}

#[test]
fn test_cube_intersection() {
    use nalgebra::Vector3;

    let cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let cube2 = cube(Vector3::new(2.0, 2.0, 2.0), true);

    // Offset second cube to partially intersect first
    let cube2_transformed = meshbool::translate(&cube2, nalgebra::Point3::new(1.0, 0.0, 0.0));

    let intersection = &cube1 ^ &cube2_transformed;
    let mesh = get_mesh_gl(&intersection, 0);

    assert!(!mesh.tri_verts.is_empty());
}
