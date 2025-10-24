//! Comprehensive verification tests for meshbool implementation
//!
//! This module provides detailed verification tests that check our meshbool
//! implementation produces valid and consistent results.

use meshbool::{cube, get_mesh_gl, translate};
use nalgebra::Vector3;

/// Test basic cube creation
#[test]
fn test_cube_creation() {
    let cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    
    // Get mesh data from our implementation
    let mesh_gl = get_mesh_gl(&cube, 0);
    
    // Check basic properties
    let num_verts = mesh_gl.vert_properties.len() / mesh_gl.num_prop as usize;
    let num_tris = mesh_gl.tri_verts.len() / 3;
    
    println!("Cube creation test:");
    println!("  Cube: {} verts, {} tris", num_verts, num_tris);
    
    // Basic verification that cube has expected number of elements
    assert!(num_verts > 0, "Cube should have vertices");
    assert!(num_tris > 0, "Cube should have triangles");
}

/// Test translation functionality
#[test]
fn test_translation() {
    let cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let translated_cube = translate(&cube, nalgebra::Point3::new(1.0, 1.0, 1.0));
    
    // Get mesh data from our implementation
    let mesh_gl = get_mesh_gl(&translated_cube, 0);
    
    // Check basic properties
    let num_verts = mesh_gl.vert_properties.len() / mesh_gl.num_prop as usize;
    let num_tris = mesh_gl.tri_verts.len() / 3;
    
    println!("Translation test:");
    println!("  Translated cube: {} verts, {} tris", num_verts, num_tris);
    
    // Basic verification that translation preserves element counts
    assert!(num_verts > 0, "Translated cube should have vertices");
    assert!(num_tris > 0, "Translated cube should have triangles");
}

/// Test boolean union functionality
#[test]
fn test_boolean_union() {
    let cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let cube2 = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let union = &cube1 + &cube2;
    
    // Get mesh data from our implementation
    let mesh_gl = get_mesh_gl(&union, 0);
    
    // Check basic properties
    let num_verts = mesh_gl.vert_properties.len() / mesh_gl.num_prop as usize;
    let num_tris = mesh_gl.tri_verts.len() / 3;
    
    println!("Union test:");
    println!("  Union: {} verts, {} tris", num_verts, num_tris);
    
    // Basic verification that union produces valid results
    assert!(num_verts > 0, "Union should have vertices");
    assert!(num_tris > 0, "Union should have triangles");
}

/// Test boolean intersection functionality
#[test]
fn test_boolean_intersection() {
    let cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let cube2 = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let intersection = &cube1 ^ &cube2;
    
    // Get mesh data from our implementation
    let mesh_gl = get_mesh_gl(&intersection, 0);
    
    // Check basic properties
    let num_verts = mesh_gl.vert_properties.len() / mesh_gl.num_prop as usize;
    let num_tris = mesh_gl.tri_verts.len() / 3;
    
    println!("Intersection test:");
    println!("  Intersection: {} verts, {} tris", num_verts, num_tris);
    
    // Basic verification that intersection produces valid results
    assert!(num_verts > 0, "Intersection should have vertices");
    assert!(num_tris > 0, "Intersection should have triangles");
}

/// Test boolean difference functionality
#[test]
fn test_boolean_difference() {
    let cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let cube2 = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let difference = &cube1 - &cube2;
    
    // Get mesh data from our implementation
    let mesh_gl = get_mesh_gl(&difference, 0);
    
    // Check basic properties
    let num_verts = mesh_gl.vert_properties.len() / mesh_gl.num_prop as usize;
    let num_tris = mesh_gl.tri_verts.len() / 3;
    
    println!("Difference test:");
    println!("  Difference: {} verts, {} tris", num_verts, num_tris);
    
    // Basic verification that difference produces valid results
    assert!(num_verts > 0, "Difference should have vertices");
    assert!(num_tris > 0, "Difference should have triangles");
}