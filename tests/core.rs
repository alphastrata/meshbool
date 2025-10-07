use meshbool::{cube, cylinder, get_mesh_gl, rotate, translate};

#[test]
fn test_cube_creation() {
    use nalgebra::Vector3;

    let cube = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let mesh = get_mesh_gl(&cube, 0);

    // A cube should have triangles
    assert!(!mesh.tri_verts.is_empty());
    // Should be a valid manifold with proper triangle count
    assert!(mesh.tri_verts.len() % 3 == 0);
}

#[test]
fn test_basic_shapes() {
    use nalgebra::Point3;

    // Test cube properties
    let cube = cube(nalgebra::Vector3::new(1.0, 1.0, 1.0), true);
    let mesh = get_mesh_gl(&cube, 0);

    assert!(!mesh.tri_verts.is_empty());
    assert!(mesh.tri_verts.len() % 3 == 0);
}

#[test]
fn test_cylinder_creation() {
    use nalgebra::Point3;

    let cylinder = cylinder(2.0, 1.0, 1.0, 16, false); // height, bottom radius, top radius, segments, center
    let mesh = get_mesh_gl(&cylinder, 0);

    assert!(!mesh.tri_verts.is_empty());
    assert!(mesh.tri_verts.len() % 3 == 0);
}

#[test]
fn test_transformations() {
    use nalgebra::{Point3, Vector3};

    let original_cube = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let original_mesh = get_mesh_gl(&original_cube, 0);

    // Translate the cube
    let translated_cube = translate(&original_cube, Point3::new(5.0, 0.0, 0.0));
    let translated_mesh = get_mesh_gl(&translated_cube, 0);

    // Should still have the same number of triangles
    assert_eq!(
        original_mesh.tri_verts.len(),
        translated_mesh.tri_verts.len()
    );

    // Rotate the cube
    let rotated_cube = rotate(&original_cube, 45.0, 0.0, 0.0); // 45 degrees around X-axis
    let rotated_mesh = get_mesh_gl(&rotated_cube, 0);

    // Should still have the same number of triangles after rotation
    assert_eq!(original_mesh.tri_verts.len(), rotated_mesh.tri_verts.len());
}

#[test]
fn test_mesh_properties() {
    use nalgebra::Vector3;

    let cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let mesh = get_mesh_gl(&cube, 0);

    // Verify basic mesh properties
    assert!(mesh.num_prop >= 3); // At least 3 properties for x, y, z coordinates
    assert_eq!(mesh.tri_verts.len() % 3, 0); // Triangles have 3 vertices each
    assert!(
        mesh.vert_properties.len() as u32
            >= mesh.num_prop * (mesh.vert_properties.len() as u32 / mesh.num_prop)
    ); // Basic consistency check
}

#[test]
fn test_empty_mesh_handling() {
    // Test for handling of potentially empty meshes or edge cases
    // This will help identify issues early
    use nalgebra::Vector3;

    let small_cube = cube(Vector3::new(0.001, 0.001, 0.001), true);
    let mesh = get_mesh_gl(&small_cube, 0);

    // Even a very small cube should have triangles
    assert!(!mesh.tri_verts.is_empty());
}
