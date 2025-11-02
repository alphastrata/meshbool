use meshbool::MeshBool;
use nalgebra::{Vector3, Point3};

#[test]
fn test_basic_cube_creation() {
    let our_cube = MeshBool::cube(Vector3::new(2.0, 2.0, 2.0), true);
    
    // Basic sanity checks for our implementation
    let mesh_gl = our_cube.get_mesh_gl(0);
    assert!(our_cube.num_tri() > 0);
    assert!(our_cube.num_vert() > 0);
    assert!(mesh_gl.tri_verts.len() > 0);
    assert!(mesh_gl.vert_properties.len() > 0);
    
    println!("Our cube has {} triangles and {} vertices", our_cube.num_tri(), our_cube.num_vert());
}

#[test]
fn test_translation() {
    let our_cube = MeshBool::cube(Vector3::new(2.0, 2.0, 2.0), true);
    let translated_cube = our_cube.translate(Point3::new(1.0, 0.0, 0.0));
    
    // Basic sanity checks
    assert!(translated_cube.num_tri() > 0);
    assert!(translated_cube.num_vert() > 0);
    
    println!("Translated cube has {} triangles and {} vertices", translated_cube.num_tri(), translated_cube.num_vert());
}

#[test]
fn test_boolean_union() {
    let our_cube1 = MeshBool::cube(Vector3::new(2.0, 2.0, 2.0), true);
    let our_cube2 = MeshBool::cube(Vector3::new(1.0, 1.0, 1.0), true);
    let our_union = &our_cube1 + &our_cube2;
    
    assert!(our_union.num_tri() > 0);
    assert!(our_union.num_vert() > 0);
    
    println!("Union operation successful for our implementation");
    println!("Our union: {} triangles", our_union.num_tri());
}

#[test]
fn test_boolean_intersection() {
    let our_cube1 = MeshBool::cube(Vector3::new(2.0, 2.0, 2.0), true);
    let our_cube2 = MeshBool::cube(Vector3::new(1.0, 1.0, 1.0), true);
    let our_intersection = &our_cube1 ^ &our_cube2;
    
    assert!(our_intersection.num_tri() > 0);
    assert!(our_intersection.num_vert() > 0);
    
    println!("Intersection operation successful for our implementation");
    println!("Our intersection: {} triangles", our_intersection.num_tri());
}

#[test]
fn test_boolean_difference() {
    let our_cube1 = MeshBool::cube(Vector3::new(2.0, 2.0, 2.0), true);
    let our_cube2 = MeshBool::cube(Vector3::new(1.0, 1.0, 1.0), true);
    let our_difference = &our_cube1 - &our_cube2;
    
    assert!(our_difference.num_tri() > 0);
    assert!(our_difference.num_vert() > 0);
    
    println!("Difference operation successful for our implementation");
    println!("Our difference: {} triangles", our_difference.num_tri());
}