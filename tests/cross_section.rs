use meshbool::{cross_section, cube};
use nalgebra::Vector3;

#[test]
fn test_cross_section_basic() {
    let cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    // Test cross-section at height 0 (middle of cube)
    let section = cross_section(&cube, 0.0);
    
    // Should produce a valid cross-section with triangles and vertices
    assert!(section.num_tri() > 0);
    assert!(section.num_vert() > 0);
    
    // For a 2x2x2 cube, cross-section at z=0 should produce a square
    // Which should have 2 triangles and 4 vertices minimum
    assert!(section.num_tri() >= 2);
    assert!(section.num_vert() >= 4);
}

#[test]
fn test_cross_section_with_height() {
    // Cross section at specific height (not at boundary)
    let cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let section = cross_section(&cube, 0.5);
    
    // Should produce a valid cross-section
    assert!(section.num_tri() > 0);
    assert!(section.num_vert() > 0);
}

#[test]
fn test_cross_section_edge_cases() {
    // Edge cases for cross section
    
    // Test with very small cube
    let small_cube = cube(Vector3::new(0.001, 0.001, 0.001), true);
    let section = cross_section(&small_cube, 0.0);
    // Should produce a valid cross-section
    assert!(section.num_tri() > 0);
    assert!(section.num_vert() > 0);
    
    // Test with different sized cube
    let medium_cube = cube(Vector3::new(5.0, 3.0, 7.0), true);
    let section = cross_section(&medium_cube, 1.0);
    // Should produce a valid cross-section
    assert!(section.num_tri() > 0);
    assert!(section.num_vert() > 0);
}

#[test]
fn test_cross_section_complex_shape() {
    // Cross section of complex boolean result
    let cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let cube2 = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let complex = &cube1 + &cube2;
    let section = cross_section(&complex, 0.5);
    
    // Should not panic when computing cross-section
    // Result may vary based on boolean operation - testing that it completes successfully
    
    // Test should at least complete without panicking
    let _tri_count = section.num_tri();
    let _vert_count = section.num_vert();
}

#[test]
fn test_cross_section_large_shape() {
    // Cross section of large shape
    let large_cube = cube(Vector3::new(100.0, 100.0, 100.0), true);
    let section = cross_section(&large_cube, 25.0);
    
    // Should produce a valid cross-section
    assert!(section.num_tri() > 0);
    assert!(section.num_vert() > 0);
    
    // Large cube cross-section should have substantial geometry
    assert!(section.num_tri() >= 2);
    assert!(section.num_vert() >= 4);
}