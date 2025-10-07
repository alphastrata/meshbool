use meshbool::{cube, get_mesh_gl, cylinder, translate};
use nalgebra::{Vector3, Point3};

#[test]
fn test_polygon_basic_functionality() {
    // Basic test to ensure polygons work as expected with existing cube functionality
    let cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let mesh = get_mesh_gl(&cube, 0);
    
    // Basic polygon validation
    assert!(mesh.tri_verts.len() % 3 == 0); // Triangles have 3 vertices
    assert!(mesh.num_prop >= 3); // At least x, y, z coordinates
    let num_verts = mesh.vert_properties.len() / mesh.num_prop as usize;
    assert!(num_verts > 0); // Should have at least some vertices
}

#[test]
fn test_polygon_overlap_and_intersection() {
    // Test with overlapping shapes to test underlying polygon operations
    let cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let cube2 = translate(&cube1, Point3::new(1.0, 0.0, 0.0)); // Translate to create overlap
    
    // This will exercise polygon intersection functionality internally
    let intersection = &cube1 ^ &cube2;
    let mesh = get_mesh_gl(&intersection, 0);
    
    assert!(!mesh.tri_verts.is_empty());
    
    // Intersection should have fewer triangles than individual cubes
    let cube1_mesh = get_mesh_gl(&cube1, 0);
    assert!(mesh.tri_verts.len() <= cube1_mesh.tri_verts.len());
}

#[test]
fn test_polygon_cylinder() {
    // Test cylinder which involves polygon operations
    let cyl = cylinder(2.0, 1.0, 1.0, 16, true); // height, radius_low, radius_high, segments, center
    let mesh = get_mesh_gl(&cyl, 0);
    
    assert!(!mesh.tri_verts.is_empty());
    assert!(mesh.tri_verts.len() % 3 == 0); // Each triangle has 3 vertices
}

#[test]
fn test_polygon_edge_cases() {
    // Test very small polygons
    let tiny_cube = cube(Vector3::new(0.001, 0.001, 0.001), true);
    let mesh = get_mesh_gl(&tiny_cube, 0);
    
    assert!(!mesh.tri_verts.is_empty());
    
    // Test large polygons
    let large_cube = cube(Vector3::new(100.0, 100.0, 100.0), true);
    let large_mesh = get_mesh_gl(&large_cube, 0);
    
    assert!(!large_mesh.tri_verts.is_empty());
    assert!(large_mesh.tri_verts.len() % 3 == 0);
}

#[test]
fn test_polygon_properties() {
    // Test that polygon properties are maintained properly
    let cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let mesh = get_mesh_gl(&cube, 0);
    
    // Should have position properties (at least 3 per vertex: x, y, z)
    assert!(mesh.num_prop >= 3);
    
    // Number of vertices should match the property buffer
    let expected_verts = mesh.vert_properties.len() / mesh.num_prop as usize;
    assert!(expected_verts > 0);
    
    // Each triangle references existing vertices
    for &vert_idx in &mesh.tri_verts {
        assert!((vert_idx as usize) < expected_verts);
    }
}