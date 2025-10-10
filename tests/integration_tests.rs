use meshbool::{cube, get_mesh_gl, translate};
use manifold_rs::Manifold;
use nalgebra::Vector3;

#[test]
fn test_cube_comparison() {
    // Create identical cubes in both implementations
    let our_cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let their_cube = Manifold::cube(2.0, 2.0, 2.0);
    
    // Basic sanity checks for our implementation
    let mesh_gl = get_mesh_gl(&our_cube, 0);
    assert!(our_cube.num_tri() > 0);
    assert!(our_cube.num_vert() > 0);
    assert!(mesh_gl.vert_properties.len() > 0);
    assert!(mesh_gl.tri_verts.len() > 0);
    
    // Basic sanity checks for manifold-rs implementation
    let their_mesh = their_cube.to_mesh();
    assert!(their_mesh.vertices().len() > 0);
    assert!(their_mesh.indices().len() > 0);
    
    let their_num_verts = their_mesh.vertices().len() / 3;
    let their_num_tris = their_mesh.indices().len() / 3;
    
    println!("Our cube: {} triangles, {} vertices", our_cube.num_tri(), our_cube.num_vert());
    println!("Their cube: {} triangles, {} vertices", their_num_tris, their_num_verts);
    
    // Compare vertex counts
    let our_num_verts = mesh_gl.vert_properties.len() / mesh_gl.num_prop as usize;
    let our_num_tris = mesh_gl.tri_verts.len() / 3;
    
    println!("Our cube mesh: {} verts, {} tris", our_num_verts, our_num_tris);
    println!("Their cube mesh: {} verts, {} tris", their_num_verts, their_num_tris);
    
    // Basic checks that both implementations produce reasonable results
    assert!(our_num_verts > 0);
    assert!(their_num_verts > 0);
    assert!(our_num_tris > 0);
    assert!(their_num_tris > 0);
}

#[test]
fn test_translation_comparison() {
    // Create and translate identical cubes in both implementations
    let our_cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let translated_our_cube = translate(&our_cube, nalgebra::Point3::new(1.0, 1.0, 1.0));
    
    let their_cube = Manifold::cube(2.0, 2.0, 2.0);
    let translated_their_cube = their_cube.translate(1.0, 1.0, 1.0);
    
    // Both should produce valid results
    assert!(translated_our_cube.num_tri() > 0);
    assert!(translated_our_cube.num_vert() > 0);
    
    let their_mesh = translated_their_cube.to_mesh();
    let their_num_verts = their_mesh.vertices().len() / 3;
    let their_num_tris = their_mesh.indices().len() / 3;
    assert!(their_num_tris > 0);
    assert!(their_num_verts > 0);
    
    println!("Translation successful for both implementations");
    println!("Our translated cube: {} triangles", translated_our_cube.num_tri());
    println!("Their translated cube: {} triangles", their_num_tris);
}

#[test]
fn test_boolean_union_comparison() {
    // Create identical boolean unions in both implementations
    let our_cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let our_cube2 = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let our_union = &our_cube1 + &our_cube2;
    
    let their_cube1 = Manifold::cube(2.0, 2.0, 2.0);
    let their_cube2 = Manifold::cube(1.0, 1.0, 1.0);
    let their_union = their_cube1.union(&their_cube2);
    
    // Both should produce valid results
    assert!(our_union.num_tri() > 0);
    assert!(our_union.num_vert() > 0);
    
    let their_mesh = their_union.to_mesh();
    let their_num_verts = their_mesh.vertices().len() / 3;
    let their_num_tris = their_mesh.indices().len() / 3;
    assert!(their_num_tris > 0);
    assert!(their_num_verts > 0);
    
    println!("Union operation successful for both implementations");
    println!("Our union: {} triangles", our_union.num_tri());
    println!("Their union: {} triangles", their_num_tris);
    
    // Basic consistency check
    assert!(our_union.num_tri() > 0);
    assert!(their_num_tris > 0);
}

#[test]
fn test_boolean_intersection_comparison() {
    // Create identical boolean intersections in both implementations
    let our_cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let our_cube2 = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let our_intersection = &our_cube1 ^ &our_cube2;
    
    let their_cube1 = Manifold::cube(2.0, 2.0, 2.0);
    let their_cube2 = Manifold::cube(1.0, 1.0, 1.0);
    let their_intersection = their_cube1.intersection(&their_cube2);
    
    // Both should produce valid results
    assert!(our_intersection.num_tri() > 0);
    assert!(our_intersection.num_vert() > 0);
    
    let their_mesh = their_intersection.to_mesh();
    let their_num_verts = their_mesh.vertices().len() / 3;
    let their_num_tris = their_mesh.indices().len() / 3;
    assert!(their_num_tris > 0);
    assert!(their_num_verts > 0);
    
    println!("Intersection operation successful for both implementations");
    println!("Our intersection: {} triangles", our_intersection.num_tri());
    println!("Their intersection: {} triangles", their_num_tris);
    
    // Basic consistency check
    assert!(our_intersection.num_tri() > 0);
    assert!(their_num_tris > 0);
}

#[test]
fn test_boolean_difference_comparison() {
    // Create identical boolean differences in both implementations
    let our_cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let our_cube2 = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let our_difference = &our_cube1 - &our_cube2;
    
    let their_cube1 = Manifold::cube(2.0, 2.0, 2.0);
    let their_cube2 = Manifold::cube(1.0, 1.0, 1.0);
    let their_difference = their_cube1.difference(&their_cube2);
    
    // Both should produce valid results
    assert!(our_difference.num_tri() > 0);
    assert!(our_difference.num_vert() > 0);
    
    let their_mesh = their_difference.to_mesh();
    let their_num_verts = their_mesh.vertices().len() / 3;
    let their_num_tris = their_mesh.indices().len() / 3;
    assert!(their_num_tris > 0);
    assert!(their_num_verts > 0);
    
    println!("Difference operation successful for both implementations");
    println!("Our difference: {} triangles", our_difference.num_tri());
    println!("Their difference: {} triangles", their_num_tris);
    
    // Basic consistency check
    assert!(our_difference.num_tri() > 0);
    assert!(their_num_tris > 0);
}