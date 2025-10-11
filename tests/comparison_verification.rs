//! Comprehensive verification tests comparing meshbool with manifold-rs
//!
//! This module provides detailed comparison tests that verify our meshbool
//! implementation produces results that are approximately equal to the 
//! original manifold-rs library.

use meshbool::{cube, get_mesh_gl, translate};
use manifold_rs::Manifold;
use nalgebra::Vector3;

/// Test basic cube creation equivalence
#[test]
fn test_cube_creation_equivalence() {
    let our_cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let their_cube = Manifold::cube(2.0, 2.0, 2.0);
    
    // Get mesh data from both implementations
    let our_mesh_gl = get_mesh_gl(&our_cube, 0);
    let their_mesh_gl = their_cube.to_mesh();
    
    // Compare basic properties
    let our_num_verts = our_mesh_gl.vert_properties.len() / our_mesh_gl.num_prop as usize;
    let their_num_verts = their_mesh_gl.vertices().len() / 3;
    
    let our_num_tris = our_mesh_gl.tri_verts.len() / 3;
    let their_num_tris = their_mesh_gl.indices().len() / 3;
    
    println!("Cube creation equivalence:");
    println!("  Our cube: {} verts, {} tris", our_num_verts, our_num_tris);
    println!("  Their cube: {} verts, {} tris", their_num_verts, their_num_tris);
    
    // Allow for some variation due to different triangulation strategies
    let vert_diff = (our_num_verts as i32 - their_num_verts as i32).abs();
    let tri_diff = (our_num_tris as i32 - their_num_tris as i32).abs();
    
    // For a cube, we expect roughly the same number of vertices and triangles
    assert!(vert_diff <= 2, "Vertex count difference should be small: {} vs {}", our_num_verts, their_num_verts);
    assert!(tri_diff <= 2, "Triangle count difference should be small: {} vs {}", our_num_tris, their_num_tris);
}

/// Test translation equivalence
#[test]
fn test_translation_equivalence() {
    let our_cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let translated_our_cube = translate(&our_cube, nalgebra::Point3::new(1.0, 1.0, 1.0));
    
    let their_cube = Manifold::cube(2.0, 2.0, 2.0);
    let translated_their_cube = their_cube.translate(1.0, 1.0, 1.0);
    
    // Get mesh data from both implementations
    let our_mesh_gl = get_mesh_gl(&translated_our_cube, 0);
    let their_mesh_gl = translated_their_cube.to_mesh();
    
    // Compare basic properties
    let our_num_verts = our_mesh_gl.vert_properties.len() / our_mesh_gl.num_prop as usize;
    let their_num_verts = their_mesh_gl.vertices().len() / 3;
    
    let our_num_tris = our_mesh_gl.tri_verts.len() / 3;
    let their_num_tris = their_mesh_gl.indices().len() / 3;
    
    println!("Translation equivalence:");
    println!("  Our translated cube: {} verts, {} tris", our_num_verts, our_num_tris);
    println!("  Their translated cube: {} verts, {} tris", their_num_verts, their_num_tris);
    
    // For translation, we expect exactly the same number of vertices and triangles
    assert_eq!(our_num_verts, their_num_verts, "Translated cubes should have same vertex count");
    assert_eq!(our_num_tris, their_num_tris, "Translated cubes should have same triangle count");
}

/// Test boolean union equivalence
#[test]
fn test_boolean_union_equivalence() {
    let our_cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let our_cube2 = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let our_union = &our_cube1 + &our_cube2;
    
    let their_cube1 = Manifold::cube(2.0, 2.0, 2.0);
    let their_cube2 = Manifold::cube(1.0, 1.0, 1.0);
    let their_union = their_cube1.union(&their_cube2);
    
    // Get mesh data from both implementations
    let our_mesh_gl = get_mesh_gl(&our_union, 0);
    let their_mesh_gl = their_union.to_mesh();
    
    // Compare basic properties
    let our_num_verts = our_mesh_gl.vert_properties.len() / our_mesh_gl.num_prop as usize;
    let their_num_verts = their_mesh_gl.vertices().len() / 3;
    
    let our_num_tris = our_mesh_gl.tri_verts.len() / 3;
    let their_num_tris = their_mesh_gl.indices().len() / 3;
    
    println!("Union equivalence:");
    println!("  Our union: {} verts, {} tris", our_num_verts, our_num_tris);
    println!("  Their union: {} verts, {} tris", their_num_verts, their_num_tris);
    
    // Allow for some variation due to different triangulation strategies
    let vert_diff = (our_num_verts as i32 - their_num_verts as i32).abs();
    let tri_diff = (our_num_tris as i32 - their_num_tris as i32).abs();
    
    // For union operations, allow more variation (up to 10% difference)
    let max_allowed_vert_diff = (their_num_verts as f64 * 0.1) as i32;
    let max_allowed_tri_diff = (their_num_tris as f64 * 0.1) as i32;
    
    assert!(vert_diff <= max_allowed_vert_diff.max(5));
    assert!(tri_diff <= max_allowed_tri_diff.max(5));
}

/// Test boolean intersection equivalence
#[test]
fn test_boolean_intersection_equivalence() {
    let our_cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let our_cube2 = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let our_intersection = &our_cube1 ^ &our_cube2;
    
    let their_cube1 = Manifold::cube(2.0, 2.0, 2.0);
    let their_cube2 = Manifold::cube(1.0, 1.0, 1.0);
    let their_intersection = their_cube1.intersection(&their_cube2);
    
    // Get mesh data from both implementations
    let our_mesh_gl = get_mesh_gl(&our_intersection, 0);
    let their_mesh_gl = their_intersection.to_mesh();
    
    // Compare basic properties
    let our_num_verts = our_mesh_gl.vert_properties.len() / our_mesh_gl.num_prop as usize;
    let their_num_verts = their_mesh_gl.vertices().len() / 3;
    
    let our_num_tris = our_mesh_gl.tri_verts.len() / 3;
    let their_num_tris = their_mesh_gl.indices().len() / 3;
    
    println!("Intersection equivalence:");
    println!("  Our intersection: {} verts, {} tris", our_num_verts, our_num_tris);
    println!("  Their intersection: {} verts, {} tris", their_num_verts, their_num_tris);
    
    // Allow for some variation due to different triangulation strategies
    let vert_diff = (our_num_verts as i32 - their_num_verts as i32).abs();
    let tri_diff = (our_num_tris as i32 - their_num_tris as i32).abs();
    
    // For intersection operations, allow moderate variation (up to 5% difference)
    let max_allowed_vert_diff = (their_num_verts as f64 * 0.05) as i32;
    let max_allowed_tri_diff = (their_num_tris as f64 * 0.05) as i32;
    
    assert!(vert_diff <= max_allowed_vert_diff.max(3));
    assert!(tri_diff <= max_allowed_tri_diff.max(3));
}

/// Test boolean difference equivalence
#[test]
fn test_boolean_difference_equivalence() {
    let our_cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let our_cube2 = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let our_difference = &our_cube1 - &our_cube2;
    
    let their_cube1 = Manifold::cube(2.0, 2.0, 2.0);
    let their_cube2 = Manifold::cube(1.0, 1.0, 1.0);
    let their_difference = their_cube1.difference(&their_cube2);
    
    // Get mesh data from both implementations
    let our_mesh_gl = get_mesh_gl(&our_difference, 0);
    let their_mesh_gl = their_difference.to_mesh();
    
    // Compare basic properties
    let our_num_verts = our_mesh_gl.vert_properties.len() / our_mesh_gl.num_prop as usize;
    let their_num_verts = their_mesh_gl.vertices().len() / 3;
    
    let our_num_tris = our_mesh_gl.tri_verts.len() / 3;
    let their_num_tris = their_mesh_gl.indices().len() / 3;
    
    println!("Difference equivalence:");
    println!("  Our difference: {} verts, {} tris", our_num_verts, our_num_tris);
    println!("  Their difference: {} verts, {} tris", their_num_verts, their_num_tris);
    
    // Allow for some variation due to different triangulation strategies
    let vert_diff = (our_num_verts as i32 - their_num_verts as i32).abs();
    let tri_diff = (our_num_tris as i32 - their_num_tris as i32).abs();
    
    // For difference operations, allow more variation (up to 15% difference)
    let max_allowed_vert_diff = (their_num_verts as f64 * 0.15) as i32;
    let max_allowed_tri_diff = (their_num_tris as f64 * 0.15) as i32;
    
    assert!(vert_diff <= max_allowed_vert_diff.max(10));
    assert!(tri_diff <= max_allowed_tri_diff.max(10));
}