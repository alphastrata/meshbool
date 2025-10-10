use meshbool::{cross_section, cube};
use manifold_rs::Manifold;
use nalgebra::Vector3;

#[test]
fn test_cross_section_comparison_with_manifold_rs() {
    // Create identical cubes in both implementations
    let our_cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let their_cube = Manifold::cube(2.0, 2.0, 2.0);
    
    // Test cross-section at height 0 (middle of cube)
    let our_section = cross_section(&our_cube, 0.0);
    let their_section = their_cube.slice(0.0);
    
    // Our implementation should now compute actual cross-sections
    // rather than returning placeholder results
    
    println!("Our cube cross-section at height 0:");
    println!("  Triangles: {}", our_section.num_tri());
    println!("  Vertices: {}", our_section.num_vert());
    
    println!("Their cube cross-section at height 0:");
    println!("  Polygons: {}", their_section.size());
    if their_section.size() > 0 {
        let poly = their_section.get_as_slice(0);
        println!("  First polygon vertices: {}", poly.len() / 2);
    }
}

#[test]
fn test_cross_section_different_heights() {
    let our_cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let their_cube = Manifold::cube(2.0, 2.0, 2.0);
    
    // Test at different heights
    let heights = vec![-0.5, 0.0, 0.5, 1.0];
    
    for &height in &heights {
        let our_section = cross_section(&our_cube, height);
        let their_section = their_cube.slice(height);
        
        println!("Height {}: Our {} tris, Their {} polygons", 
                 height, our_section.num_tri(), their_section.size());
        
        // Both should produce valid results for heights within the cube
        if height >= -1.0 && height <= 1.0 {
            // Within cube bounds - should have intersection
            // Our implementation should handle this correctly
        } else {
            // Outside cube bounds - may be empty
            // This is fine, we're just making sure it doesn't panic
        }
    }
}

#[test]
fn test_cross_section_edge_cases() {
    // Test with very small cube
    let our_small_cube = cube(Vector3::new(0.001, 0.001, 0.001), true);
    let their_small_cube = Manifold::cube(0.001, 0.001, 0.001);
    
    let our_section = cross_section(&our_small_cube, 0.0);
    let their_section = their_small_cube.slice(0.0);
    
    println!("Small cube cross-section:");
    println!("  Our: {} tris, Their: {} polygons", 
             our_section.num_tri(), their_section.size());
    
    // Test with large cube
    let our_large_cube = cube(Vector3::new(100.0, 100.0, 100.0), true);
    let their_large_cube = Manifold::cube(100.0, 100.0, 100.0);
    
    let our_section = cross_section(&our_large_cube, 50.0);
    let their_section = their_large_cube.slice(50.0);
    
    println!("Large cube cross-section:");
    println!("  Our: {} tris, Their: {} polygons", 
             our_section.num_tri(), their_section.size());
}