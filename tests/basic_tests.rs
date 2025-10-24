use meshbool::{cube, get_mesh_gl, translate};
use nalgebra::Vector3;

#[test]
fn test_basic_cube_creation() {
    let our_cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    
    // Basic sanity checks for our implementation
    let mesh_gl = get_mesh_gl(&our_cube, 0);
    assert!(our_cube.num_tri() > 0);
    assert!(our_cube.num_vert() > 0);
    assert!(mesh_gl.tri_verts.len() > 0);
    assert!(mesh_gl.vert_properties.len() > 0);
    
    println!("Our cube has {} triangles and {} vertices", our_cube.num_tri(), our_cube.num_vert());
}

#[test]
fn test_translation() {
    let our_cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let translated_our_cube = translate(&our_cube, nalgebra::Point3::new(1.0, 1.0, 1.0));
    
    assert!(translated_our_cube.num_tri() > 0);
    assert!(translated_our_cube.num_vert() > 0);
    
    println!("Translation successful for our implementation");
}

#[test]
fn test_boolean_union() {
    let our_cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let our_cube2 = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let our_union = &our_cube1 + &our_cube2;
    
    assert!(our_union.num_tri() > 0);
    assert!(our_union.num_vert() > 0);
    
    println!("Union operation successful for our implementation");
    println!("Our union: {} triangles", our_union.num_tri());
}

#[test]
fn test_boolean_intersection() {
    let our_cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let our_cube2 = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let our_intersection = &our_cube1 ^ &our_cube2;
    
    assert!(our_intersection.num_tri() > 0);
    assert!(our_intersection.num_vert() > 0);
    
    println!("Intersection operation successful for our implementation");
    println!("Our intersection: {} triangles", our_intersection.num_tri());
}

#[test]
fn test_boolean_difference() {
    let our_cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let our_cube2 = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let our_difference = &our_cube1 - &our_cube2;
    
    assert!(our_difference.num_tri() > 0);
    assert!(our_difference.num_vert() > 0);
    
    println!("Difference operation successful for our implementation");
    println!("Our difference: {} triangles", our_difference.num_tri());
}