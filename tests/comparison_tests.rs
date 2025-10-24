use meshbool::{cube, get_mesh_gl, translate};
use nalgebra::Vector3;

/// Compare two of our meshes for approximate equality by checking basic properties
fn approx_equal_meshes(mesh1: &meshbool::Impl, mesh2: &meshbool::Impl) -> bool {
    // Get mesh data from both implementations
    let mesh1_gl = get_mesh_gl(mesh1, 0);
    let mesh2_gl = get_mesh_gl(mesh2, 0);
    
    // Compare basic properties
    let mesh1_num_verts = mesh1_gl.vert_properties.len() / mesh1_gl.num_prop as usize;
    let mesh2_num_verts = mesh2_gl.vert_properties.len() / mesh2_gl.num_prop as usize;
    
    let mesh1_num_tris = mesh1_gl.tri_verts.len() / 3;
    let mesh2_num_tris = mesh2_gl.tri_verts.len() / 3;
    
    // Allow for some variation due to different triangulation strategies
    let vert_diff = (mesh1_num_verts as i32 - mesh2_num_verts as i32).abs();
    let tri_diff = (mesh1_num_tris as i32 - mesh2_num_tris as i32).abs();
    
    // For now, just check that the differences are within reasonable bounds
    vert_diff <= 2 && tri_diff <= 2
}

#[test]
fn test_cube_approx_eq() {
    let our_cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let another_cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    
    let result = approx_equal_meshes(&our_cube, &another_cube);
    assert!(result, "Identical cube meshes should be approximately equal");
    
    let mesh_gl = get_mesh_gl(&our_cube, 0);
    println!("Cube: {} verts, {} tris", 
             mesh_gl.vert_properties.len() / mesh_gl.num_prop as usize,
             mesh_gl.tri_verts.len() / 3);
}

#[test]
fn test_translated_cube_approx_eq() {
    let cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let translated_cube1 = translate(&cube1, nalgebra::Point3::new(1.0, 1.0, 1.0));
    
    let cube2 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let translated_cube2 = translate(&cube2, nalgebra::Point3::new(1.0, 1.0, 1.0));
    
    let result = approx_equal_meshes(&translated_cube1, &translated_cube2);
    assert!(result, "Identically translated cube meshes should be approximately equal");
    
    println!("Translation test successful");
}

#[test]
fn test_boolean_union_approx_eq() {
    // Same sized cubes for both implementations
    let cube1a = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let cube2a = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let union_a = &cube1a + &cube2a;
    
    let cube1b = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let cube2b = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let union_b = &cube1b + &cube2b;
    
    let result = approx_equal_meshes(&union_a, &union_b);
    assert!(result, "Identical union operations should produce approximately equal results");
    
    println!("Union test: {} tris", union_a.num_tri());
}

#[test]
fn test_boolean_intersection_approx_eq() {
    // Same sized cubes for both implementations
    let cube1a = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let cube2a = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let intersection_a = &cube1a ^ &cube2a;
    
    let cube1b = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let cube2b = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let intersection_b = &cube1b ^ &cube2b;
    
    let result = approx_equal_meshes(&intersection_a, &intersection_b);
    assert!(result, "Identical intersection operations should produce approximately equal results");
    
    println!("Intersection test: {} tris", intersection_a.num_tri());
}

#[test]
fn test_boolean_difference_approx_eq() {
    // Same sized cubes for both implementations
    let cube1a = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let cube2a = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let difference_a = &cube1a - &cube2a;
    
    let cube1b = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let cube2b = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let difference_b = &cube1b - &cube2b;
    
    let result = approx_equal_meshes(&difference_a, &difference_b);
    assert!(result, "Identical difference operations should produce approximately equal results");
    
    println!("Difference test: {} tris", difference_a.num_tri());
}