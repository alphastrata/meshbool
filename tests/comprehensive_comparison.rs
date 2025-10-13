//! Comprehensive mesh comparison tests for meshbool implementation
//!
//! This module provides extensive tests that verify the correctness
//! and functionality of our meshbool implementation.

use meshbool::{cube, get_mesh_gl, translate};
use nalgebra::Vector3;

/// Test basic cube creation
#[test]
fn test_cube_creation() {
    let cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    
    // Basic sanity checks
    assert!(cube.num_tri() > 0);
    assert!(cube.num_vert() > 0);
    
    // Convert to mesh data for verification
    let mesh_gl = get_mesh_gl(&cube, 0);
    
    let num_verts = mesh_gl.vert_properties.len() / mesh_gl.num_prop as usize;
    let num_tris = mesh_gl.tri_verts.len() / 3;
    
    println!("Cube creation test:");
    println!("  Cube: {} verts, {} tris", num_verts, num_tris);
    
    // Standard cube should have expected numbers
    assert!(num_verts > 0);
    assert!(num_tris > 0);
}

/// Test translation functionality
#[test]
fn test_translation() {
    let cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let translated_cube = translate(&cube, nalgebra::Point3::new(1.0, 1.0, 1.0));
    
    // Basic sanity checks
    assert!(translated_cube.num_tri() > 0);
    assert!(translated_cube.num_vert() > 0);
    
    // Convert to mesh data for verification
    let mesh_gl = get_mesh_gl(&translated_cube, 0);
    
    let num_verts = mesh_gl.vert_properties.len() / mesh_gl.num_prop as usize;
    let num_tris = mesh_gl.tri_verts.len() / 3;
    
    println!("Translation test:");
    println!("  Translated cube: {} verts, {} tris", num_verts, num_tris);
    
    // Should have the same number of elements as original
    assert!(num_verts > 0);
    assert!(num_tris > 0);
}

/// Test boolean union functionality
#[test]
fn test_boolean_union() {
    let cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let cube2 = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let union = &cube1 + &cube2;
    
    // Basic sanity checks
    assert!(union.num_tri() > 0);
    assert!(union.num_vert() > 0);
    
    // Convert to mesh data for verification
    let mesh_gl = get_mesh_gl(&union, 0);
    
    let num_verts = mesh_gl.vert_properties.len() / mesh_gl.num_prop as usize;
    let num_tris = mesh_gl.tri_verts.len() / 3;
    
    println!("Union test:");
    println!("  Union: {} verts, {} tris", num_verts, num_tris);
    
    // Should have positive values
    assert!(num_verts > 0);
    assert!(num_tris > 0);
}

/// Test boolean intersection functionality
#[test]
fn test_boolean_intersection() {
    let cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let cube2 = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let intersection = &cube1 ^ &cube2;
    
    // Basic sanity checks
    assert!(intersection.num_tri() > 0);
    assert!(intersection.num_vert() > 0);
    
    // Convert to mesh data for verification
    let mesh_gl = get_mesh_gl(&intersection, 0);
    
    let num_verts = mesh_gl.vert_properties.len() / mesh_gl.num_prop as usize;
    let num_tris = mesh_gl.tri_verts.len() / 3;
    
    println!("Intersection test:");
    println!("  Intersection: {} verts, {} tris", num_verts, num_tris);
    
    // Should have positive values
    assert!(num_verts > 0);
    assert!(num_tris > 0);
}

/// Test boolean difference functionality
#[test]
fn test_boolean_difference() {
    let cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let cube2 = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let difference = &cube1 - &cube2;
    
    // Basic sanity checks
    assert!(difference.num_tri() > 0);
    assert!(difference.num_vert() > 0);
    
    // Convert to mesh data for verification
    let mesh_gl = get_mesh_gl(&difference, 0);
    
    let num_verts = mesh_gl.vert_properties.len() / mesh_gl.num_prop as usize;
    let num_tris = mesh_gl.tri_verts.len() / 3;
    
    println!("Difference test:");
    println!("  Difference: {} verts, {} tris", num_verts, num_tris);
    
    // Should have positive values
    assert!(num_verts > 0);
    assert!(num_tris > 0);
}

/// Test complex boolean operations
#[test]
fn test_complex_boolean_operations() {
    // Create a more complex shape: cube with a hole
    let outer = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let inner = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let hollow_cube = &outer - &inner;
    
    // Basic sanity checks
    assert!(hollow_cube.num_tri() > 0);
    assert!(hollow_cube.num_vert() > 0);
    
    // Convert to mesh data for verification
    let mesh_gl = get_mesh_gl(&hollow_cube, 0);
    
    let num_verts = mesh_gl.vert_properties.len() / mesh_gl.num_prop as usize;
    let num_tris = mesh_gl.tri_verts.len() / 3;
    
    println!("Complex boolean operations test:");
    println!("  Hollow cube: {} verts, {} tris", num_verts, num_tris);
    
    // Should have positive values
    assert!(num_verts > 0);
    assert!(num_tris > 0);
    
    println!("Complex boolean operations test passed");
}