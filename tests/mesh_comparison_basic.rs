use meshbool::{cube, get_mesh_gl};
use manifold_rs::Manifold;
use nalgebra::Vector3;

#[test]
fn test_mesh_comparison_basic() {
    // Create identical cubes in both implementations
    let our_cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let their_cube = Manifold::cube(2.0, 2.0, 2.0);
    
    // Basic sanity checks
    let our_mesh_gl = get_mesh_gl(&our_cube, 0);
    let their_mesh_gl = their_cube.to_mesh();
    
    // Check that both meshes have vertices
    assert!(!our_mesh_gl.vert_properties.is_empty());
    assert!(!their_mesh_gl.vertices().is_empty());
    
    // Check that both meshes have triangles
    assert!(!our_mesh_gl.tri_verts.is_empty());
    assert!(!their_mesh_gl.indices().is_empty());
    
    println!("Basic mesh comparison test passed");
    println!("  Our cube: {} verts, {} tris", 
             our_mesh_gl.vert_properties.len() / our_mesh_gl.num_prop as usize,
             our_mesh_gl.tri_verts.len() / 3);
    println!("  Their cube: {} verts, {} tris", 
             their_mesh_gl.vertices().len() / 3,
             their_mesh_gl.indices().len() / 3);
}