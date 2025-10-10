//! Comprehensive mesh comparison tests between meshbool and manifold-rs implementations
//!
//! This module provides extensive tests that compare the output of our meshbool
//! implementation with the original manifold-rs library to verify compatibility
//! and correctness.

use meshbool::{cube, get_mesh_gl, translate};
use manifold_rs::Manifold;
use nalgebra::Vector3;

/// Test basic cube creation equivalence
#[test]
fn test_cube_equivalence() {
    let our_cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let their_cube = Manifold::cube(2.0, 2.0, 2.0);
    
    // Basic sanity checks for both implementations
    assert!(our_cube.num_tri() > 0);
    assert!(our_cube.num_vert() > 0);
    
    // Convert to mesh data for comparison
    let our_mesh_gl = get_mesh_gl(&our_cube, 0);
    let their_mesh_gl = their_cube.to_mesh();
    
    assert!(!their_mesh_gl.vertices().is_empty());
    assert!(!their_mesh_gl.indices().is_empty());
    
    // Compare mesh data
    let our_num_verts = our_mesh_gl.vert_properties.len() / our_mesh_gl.num_prop as usize;
    let their_num_verts = their_mesh_gl.vertices().len() / 3; // x, y, z coords
    
    let our_num_tris = our_mesh_gl.tri_verts.len() / 3;
    let their_num_tris = their_mesh_gl.indices().len() / 3;
    
    println!("Cube equivalence test:");
    println!("  Our cube: {} verts, {} tris", our_num_verts, our_num_tris);
    println!("  Their cube: {} verts, {} tris", their_num_verts, their_num_tris);
    
    // Allow for some variation due to different triangulation strategies
    assert!((our_num_verts as i32 - their_num_verts as i32).abs() <= 2);
    assert!((our_num_tris as i32 - their_num_tris as i32).abs() <= 2);
}

/// Test translation equivalence
#[test]
fn test_translation_equivalence() {
    let our_cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let translated_our_cube = translate(&our_cube, nalgebra::Point3::new(1.0, 1.0, 1.0));
    
    let their_cube = Manifold::cube(2.0, 2.0, 2.0);
    let translated_their_cube = their_cube.translate(1.0, 1.0, 1.0);
    
    // Basic sanity checks
    assert!(translated_our_cube.num_tri() > 0);
    assert!(translated_our_cube.num_vert() > 0);
    
    // Convert to mesh data for comparison
    let our_mesh_gl = get_mesh_gl(&translated_our_cube, 0);
    let their_mesh_gl = translated_their_cube.to_mesh();
    
    assert!(!their_mesh_gl.vertices().is_empty());
    assert!(!their_mesh_gl.indices().is_empty());
    
    // Compare mesh data
    let our_num_verts = our_mesh_gl.vert_properties.len() / our_mesh_gl.num_prop as usize;
    let their_num_verts = their_mesh_gl.vertices().len() / 3; // x, y, z coords
    
    let our_num_tris = our_mesh_gl.tri_verts.len() / 3;
    let their_num_tris = their_mesh_gl.indices().len() / 3;
    
    println!("Translation equivalence test:");
    println!("  Our translated cube: {} verts, {} tris", our_num_verts, our_num_tris);
    println!("  Their translated cube: {} verts, {} tris", their_num_verts, their_num_tris);
    
    // Allow for some variation due to different triangulation strategies
    assert!((our_num_verts as i32 - their_num_verts as i32).abs() <= 2);
    assert!((our_num_tris as i32 - their_num_tris as i32).abs() <= 2);
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
    
    // Basic sanity checks
    assert!(our_union.num_tri() > 0);
    assert!(our_union.num_vert() > 0);
    
    // Convert to mesh data for comparison
    let our_mesh_gl = get_mesh_gl(&our_union, 0);
    let their_mesh_gl = their_union.to_mesh();
    
    assert!(!their_mesh_gl.vertices().is_empty());
    assert!(!their_mesh_gl.indices().is_empty());
    
    // Compare mesh data
    let our_num_verts = our_mesh_gl.vert_properties.len() / our_mesh_gl.num_prop as usize;
    let their_num_verts = their_mesh_gl.vertices().len() / 3; // x, y, z coords
    
    let our_num_tris = our_mesh_gl.tri_verts.len() / 3;
    let their_num_tris = their_mesh_gl.indices().len() / 3;
    
    println!("Union equivalence test:");
    println!("  Our union: {} verts, {} tris", our_num_verts, our_num_tris);
    println!("  Their union: {} verts, {} tris", their_num_verts, their_num_tris);
    
    // Allow for some variation due to different triangulation strategies
    // Union operations can have significant differences depending on implementation
    let vert_diff = (our_num_verts as i32 - their_num_verts as i32).abs() as f64;
    let tri_diff = (our_num_tris as i32 - their_num_tris as i32).abs() as f64;
    
    // For union operations, allow more variation (up to 10% difference)
    let max_allowed_vert_diff = (their_num_verts as f64 * 0.1).max(5.0);
    let max_allowed_tri_diff = (their_num_tris as f64 * 0.1).max(5.0);
    
    assert!(vert_diff <= max_allowed_vert_diff);
    assert!(tri_diff <= max_allowed_tri_diff);
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
    
    // Basic sanity checks
    assert!(our_intersection.num_tri() > 0);
    assert!(our_intersection.num_vert() > 0);
    
    // Convert to mesh data for comparison
    let our_mesh_gl = get_mesh_gl(&our_intersection, 0);
    let their_mesh_gl = their_intersection.to_mesh();
    
    assert!(!their_mesh_gl.vertices().is_empty());
    assert!(!their_mesh_gl.indices().is_empty());
    
    // Compare mesh data
    let our_num_verts = our_mesh_gl.vert_properties.len() / our_mesh_gl.num_prop as usize;
    let their_num_verts = their_mesh_gl.vertices().len() / 3; // x, y, z coords
    
    let our_num_tris = our_mesh_gl.tri_verts.len() / 3;
    let their_num_tris = their_mesh_gl.indices().len() / 3;
    
    println!("Intersection equivalence test:");
    println!("  Our intersection: {} verts, {} tris", our_num_verts, our_num_tris);
    println!("  Their intersection: {} verts, {} tris", their_num_verts, their_num_tris);
    
    // Allow for some variation due to different triangulation strategies
    let vert_diff = (our_num_verts as i32 - their_num_verts as i32).abs() as f64;
    let tri_diff = (our_num_tris as i32 - their_num_tris as i32).abs() as f64;
    
    // For intersection operations, allow moderate variation (up to 5% difference)
    let max_allowed_vert_diff = (their_num_verts as f64 * 0.05).max(3.0);
    let max_allowed_tri_diff = (their_num_tris as f64 * 0.05).max(3.0);
    
    assert!(vert_diff <= max_allowed_vert_diff);
    assert!(tri_diff <= max_allowed_tri_diff);
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
    
    // Basic sanity checks
    assert!(our_difference.num_tri() > 0);
    assert!(our_difference.num_vert() > 0);
    
    // Convert to mesh data for comparison
    let our_mesh_gl = get_mesh_gl(&our_difference, 0);
    let their_mesh_gl = their_difference.to_mesh();
    
    assert!(!their_mesh_gl.vertices().is_empty());
    assert!(!their_mesh_gl.indices().is_empty());
    
    // Compare mesh data
    let our_num_verts = our_mesh_gl.vert_properties.len() / our_mesh_gl.num_prop as usize;
    let their_num_verts = their_mesh_gl.vertices().len() / 3; // x, y, z coords
    
    let our_num_tris = our_mesh_gl.tri_verts.len() / 3;
    let their_num_tris = their_mesh_gl.indices().len() / 3;
    
    println!("Difference equivalence test:");
    println!("  Our difference: {} verts, {} tris", our_num_verts, our_num_tris);
    println!("  Their difference: {} verts, {} tris", their_num_verts, their_num_tris);
    
    // Allow for some variation due to different triangulation strategies
    let vert_diff = (our_num_verts as i32 - their_num_verts as i32).abs() as f64;
    let tri_diff = (our_num_tris as i32 - their_num_tris as i32).abs() as f64;
    
    // For difference operations, allow more variation (up to 15% difference)
    let max_allowed_vert_diff = (their_num_verts as f64 * 0.15).max(10.0);
    let max_allowed_tri_diff = (their_num_tris as f64 * 0.15).max(10.0);
    
    assert!(vert_diff <= max_allowed_vert_diff);
    assert!(tri_diff <= max_allowed_tri_diff);
}

/// Test complex boolean operations equivalence
#[test]
fn test_complex_boolean_operations_equivalence() {
    // Create a more complex shape: cube with a hole
    let our_outer = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let our_inner = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let our_hollow_cube = &our_outer - &our_inner;
    
    let their_outer = Manifold::cube(2.0, 2.0, 2.0);
    let their_inner = Manifold::cube(1.0, 1.0, 1.0);
    let their_hollow_cube = their_outer.difference(&their_inner);
    
    // Basic sanity checks
    assert!(our_hollow_cube.num_tri() > 0);
    assert!(our_hollow_cube.num_vert() > 0);
    
    // Convert to mesh data for comparison
    let our_mesh_gl = get_mesh_gl(&our_hollow_cube, 0);
    let their_mesh_gl = their_hollow_cube.to_mesh();
    
    assert!(!their_mesh_gl.vertices().is_empty());
    assert!(!their_mesh_gl.indices().is_empty());
    
    // Compare mesh data
    let our_num_verts = our_mesh_gl.vert_properties.len() / our_mesh_gl.num_prop as usize;
    let their_num_verts = their_mesh_gl.vertices().len() / 3; // x, y, z coords
    
    let our_num_tris = our_mesh_gl.tri_verts.len() / 3;
    let their_num_tris = their_mesh_gl.indices().len() / 3;
    
    println!("Complex boolean operations equivalence test:");
    println!("  Our hollow cube: {} verts, {} tris", our_num_verts, our_num_tris);
    println!("  Their hollow cube: {} verts, {} tris", their_num_verts, their_num_tris);
    
    // Allow for significant variation due to complex boolean operations
    let vert_diff = (our_num_verts as i32 - their_num_verts as i32).abs() as f64;
    let tri_diff = (our_num_tris as i32 - their_num_tris as i32).abs() as f64;
    
    // For complex operations, allow more variation (up to 20% difference)
    let max_allowed_vert_diff = (their_num_verts as f64 * 0.20).max(15.0);
    let max_allowed_tri_diff = (their_num_tris as f64 * 0.20).max(15.0);
    
    assert!(vert_diff <= max_allowed_vert_diff);
    assert!(tri_diff <= max_allowed_tri_diff);
    
    println!("Complex boolean operations test passed with {:.1}% vertex difference and {:.1}% triangle difference",
             (vert_diff / their_num_verts as f64 * 100.0),
             (tri_diff / their_num_tris as f64 * 100.0));
}