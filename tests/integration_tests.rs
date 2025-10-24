use meshbool::{cube, get_mesh_gl, translate};
use nalgebra::Vector3;

#[test]
fn test_basic_cube_creation() {
    // Create cube using our implementation
    let cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    
    // Basic sanity checks for our implementation
    let mesh_gl = get_mesh_gl(&cube, 0);
    assert!(cube.num_tri() > 0);
    assert!(cube.num_vert() > 0);
    assert!(mesh_gl.vert_properties.len() > 0);
    assert!(mesh_gl.tri_verts.len() > 0);
    
    let num_verts = mesh_gl.vert_properties.len() / mesh_gl.num_prop as usize;
    let num_tris = mesh_gl.tri_verts.len() / 3;
    
    println!("Cube: {} triangles, {} vertices", cube.num_tri(), cube.num_vert());
    println!("Cube mesh: {} verts, {} tris", num_verts, num_tris);
    
    // Check that it produces reasonable results
    assert!(num_verts > 0);
    assert!(num_tris > 0);
}

#[test]
fn test_translation() {
    // Create and translate a cube in our implementation
    let cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let translated_cube = translate(&cube, nalgebra::Point3::new(1.0, 1.0, 1.0));
    
    // Should produce valid results
    assert!(translated_cube.num_tri() > 0);
    assert!(translated_cube.num_vert() > 0);
    
    println!("Translation successful");
    println!("Translated cube: {} triangles", translated_cube.num_tri());
}

#[test]
fn test_boolean_union() {
    // Create identical boolean unions in our implementation
    let cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let cube2 = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let union = &cube1 + &cube2;
    
    // Should produce valid results
    assert!(union.num_tri() > 0);
    assert!(union.num_vert() > 0);
    
    println!("Union operation successful");
    println!("Union: {} triangles", union.num_tri());
    
    // Basic consistency check
    assert!(union.num_tri() > 0);
}

#[test]
fn test_boolean_intersection() {
    // Create identical boolean intersections in our implementation
    let cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let cube2 = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let intersection = &cube1 ^ &cube2;
    
    // Should produce valid results
    assert!(intersection.num_tri() > 0);
    assert!(intersection.num_vert() > 0);
    
    println!("Intersection operation successful");
    println!("Intersection: {} triangles", intersection.num_tri());
    
    // Basic consistency check
    assert!(intersection.num_tri() > 0);
}

#[test]
fn test_boolean_difference() {
    // Create identical boolean differences in our implementation
    let cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let cube2 = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let difference = &cube1 - &cube2;
    
    // Should produce valid results
    assert!(difference.num_tri() > 0);
    assert!(difference.num_vert() > 0);
    
    println!("Difference operation successful");
    println!("Difference: {} triangles", difference.num_tri());
    
    // Basic consistency check
    assert!(difference.num_tri() > 0);
}