use meshbool::{cube, get_mesh_gl, translate};
use nalgebra::Vector3;

#[test]
fn test_basic_cube_creation() {
    let our_cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let mesh_gl = get_mesh_gl(&our_cube, 0);
    
    // Basic sanity checks
    assert!(our_cube.num_tri() > 0);
    assert!(our_cube.num_vert() > 0);
    assert!(mesh_gl.tri_verts.len() > 0);
    assert!(mesh_gl.vert_properties.len() > 0);
    println!("Cube has {} triangles and {} vertices", our_cube.num_tri(), our_cube.num_vert());
}

#[test]
fn test_translation() {
    let our_cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let translated_cube = translate(&our_cube, nalgebra::Point3::new(1.0, 1.0, 1.0));
    
    assert!(translated_cube.num_tri() > 0);
    assert!(translated_cube.num_vert() > 0);
    assert_eq!(our_cube.num_tri(), translated_cube.num_tri());
    println!("Translation successful");
}

#[test]
fn test_boolean_union() {
    let cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let cube2 = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let union_result = &cube1 + &cube2;
    
    assert!(union_result.num_tri() > 0);
    assert!(union_result.num_vert() > 0);
    println!("Union operation successful, result has {} triangles", union_result.num_tri());
}

#[test]
fn test_boolean_intersection() {
    let cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let cube2 = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let intersection_result = &cube1 ^ &cube2;
    
    assert!(intersection_result.num_tri() > 0);
    assert!(intersection_result.num_vert() > 0);
    println!("Intersection operation successful, result has {} triangles", intersection_result.num_tri());
}

#[test]
fn test_boolean_difference() {
    let cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let cube2 = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let difference_result = &cube1 - &cube2;
    
    assert!(difference_result.num_tri() > 0);
    assert!(difference_result.num_vert() > 0);
    println!("Difference operation successful, result has {} triangles", difference_result.num_tri());
}