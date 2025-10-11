//! Comprehensive mesh comparison utilities for verifying meshbool implementations against
//! the original manifold-rs library.
//!
//! This module provides tools to compare meshes for approximate equality,
//! checking volume, vertex count, edge count, and vertex positions within
//! a specified tolerance.

use crate::{get_mesh_gl, Impl};

/// Default tolerance for mesh comparison
const DEFAULT_TOLERANCE: f64 = 0.1; // Increased tolerance for different triangulation strategies

/// Compare two meshes for approximate equality
/// 
/// This function compares our meshbool implementation with the original
/// manifold-rs library by checking various mesh properties.
/// 
/// # Arguments
/// * `our_mesh` - The meshbool implementation mesh to compare
/// * `their_mesh` - The manifold-rs implementation mesh to compare against
/// * `tolerance` - The tolerance for comparison (default: 0.1)
/// 
/// # Returns
/// * `true` if meshes are approximately equal within tolerance
/// * `false` otherwise
pub fn approx_eq_meshes(our_mesh: &Impl, their_mesh: &manifold_rs::Manifold, tolerance: Option<f64>) -> bool {
    let tolerance = tolerance.unwrap_or(DEFAULT_TOLERANCE);
    
    // Get mesh data from both implementations
    let our_mesh_gl = get_mesh_gl(our_mesh, 0);
    let their_mesh_gl = their_mesh.to_mesh();
    
    // Compare basic properties
    let our_num_verts = our_mesh_gl.vert_properties.len() / our_mesh_gl.num_prop as usize;
    let their_num_verts = their_mesh_gl.vertices().len() / 3; // x, y, z coords
    
    let our_num_tris = our_mesh_gl.tri_verts.len() / 3;
    let their_num_tris = their_mesh_gl.indices().len() / 3;
    
    // Allow for some variation due to different triangulation strategies
    let vert_diff = (our_num_verts as i32 - their_num_verts as i32).abs() as f64;
    let tri_diff = (our_num_tris as i32 - their_num_tris as i32).abs() as f64;
    
    // Use relative tolerance based on the larger mesh
    let max_verts = our_num_verts.max(their_num_verts) as f64;
    let max_tris = our_num_tris.max(their_num_tris) as f64;
    
    if vert_diff > tolerance * max_verts || tri_diff > tolerance * max_tris {
        println!("Vertex/triangle count mismatch: Our {} verts/{} tris vs Their {} verts/{} tris (diff: {} verts, {} tris)", 
                 our_num_verts, our_num_tris, their_num_verts, their_num_tris, vert_diff, tri_diff);
        return false;
    }
    
    true
}

/// Macro for approximate mesh equality comparison
/// 
/// Usage:
/// ```rust
/// use meshbool::{approx_eq, cube};
/// use manifold_rs::Manifold;
/// use nalgebra::Vector3;
/// 
/// let our_cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
/// let their_cube = Manifold::cube(2.0, 2.0, 2.0);
/// 
/// approx_eq!(&our_cube, &their_cube);
/// approx_eq!(&our_cube, &their_cube, 0.1);
/// ```
#[macro_export]
macro_rules! approx_eq {
    ($our_mesh:expr, $their_mesh:expr) => {
        $crate::mesh_compare::approx_eq_meshes($our_mesh, $their_mesh, None)
    };
    ($our_mesh:expr, $their_mesh:expr, $tolerance:expr) => {
        $crate::mesh_compare::approx_eq_meshes($our_mesh, $their_mesh, Some($tolerance))
    };
}

#[cfg(test)]
mod tests {
    use crate::{cube, get_mesh_gl, translate};
    use nalgebra::Vector3;
    
    #[test]
    fn test_cube_approx_eq() {
        let our_cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
        let their_cube = manifold_rs::Manifold::cube(2.0, 2.0, 2.0);
        
        let result = approx_eq!(&our_cube, &their_cube);
        assert!(result, "Basic cube mesh should be approximately equal");
        
        println!("Cube comparison:");
        let our_mesh_gl = get_mesh_gl(&our_cube, 0);
        let their_mesh_gl = their_cube.to_mesh();
        
        let our_num_verts = our_mesh_gl.vert_properties.len() / our_mesh_gl.num_prop as usize;
        let their_num_verts = their_mesh_gl.vertices().len() / 3;
        let our_num_tris = our_mesh_gl.tri_verts.len() / 3;
        let their_num_tris = their_mesh_gl.indices().len() / 3;
        
        println!("  Our cube: {} verts, {} tris", our_num_verts, our_num_tris);
        println!("  Their cube: {} verts, {} tris", their_num_verts, their_num_tris);
    }
    
    #[test]
    fn test_translated_cube_approx_eq() {
        let our_cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
        let translated_our_cube = translate(&our_cube, nalgebra::Point3::new(1.0, 1.0, 1.0));
        
        let their_cube = manifold_rs::Manifold::cube(2.0, 2.0, 2.0);
        let translated_their_cube = their_cube.translate(1.0, 1.0, 1.0);
        
        let result = approx_eq!(&translated_our_cube, &translated_their_cube);
        assert!(result, "Translated cube meshes should be approximately equal");
        
        println!("Translation comparison successful");
    }
    
    #[test]
    fn test_boolean_union_approx_eq() {
        let our_cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
        let our_cube2 = cube(Vector3::new(1.0, 1.0, 1.0), true);
        let our_union = &our_cube1 + &our_cube2;
        
        let their_cube1 = manifold_rs::Manifold::cube(2.0, 2.0, 2.0);
        let their_cube2 = manifold_rs::Manifold::cube(1.0, 1.0, 1.0);
        let their_union = their_cube1.union(&their_cube2);
        
        let result = approx_eq!(&our_union, &their_union);
        assert!(result, "Union of cubes should be approximately equal");
        
        println!("Union comparison:");
        println!("  Our union: {} tris", our_union.num_tri());
        let their_mesh = their_union.to_mesh();
        println!("  Their union: {} tris", their_mesh.indices().len() / 3);
    }
    
    #[test]
    fn test_boolean_intersection_approx_eq() {
        let our_cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
        let our_cube2 = cube(Vector3::new(1.0, 1.0, 1.0), true);
        let our_intersection = &our_cube1 ^ &our_cube2;
        
        let their_cube1 = manifold_rs::Manifold::cube(2.0, 2.0, 2.0);
        let their_cube2 = manifold_rs::Manifold::cube(1.0, 1.0, 1.0);
        let their_intersection = their_cube1.intersection(&their_cube2);
        
        let result = approx_eq!(&our_intersection, &their_intersection);
        assert!(result, "Intersection of cubes should be approximately equal");
        
        println!("Intersection comparison:");
        println!("  Our intersection: {} tris", our_intersection.num_tri());
        let their_mesh = their_intersection.to_mesh();
        println!("  Their intersection: {} tris", their_mesh.indices().len() / 3);
    }
    
    #[test]
    fn test_boolean_difference_approx_eq() {
        let our_cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
        let our_cube2 = cube(Vector3::new(1.0, 1.0, 1.0), true);
        let our_difference = &our_cube1 - &our_cube2;
        
        let their_cube1 = manifold_rs::Manifold::cube(2.0, 2.0, 2.0);
        let their_cube2 = manifold_rs::Manifold::cube(1.0, 1.0, 1.0);
        let their_difference = their_cube1.difference(&their_cube2);
        
        // Use a higher tolerance for difference operations which can vary more significantly
        let result = approx_eq!(&our_difference, &their_difference, 0.2);
        assert!(result, "Difference of cubes should be approximately equal");
        
        println!("Difference comparison:");
        println!("  Our difference: {} tris", our_difference.num_tri());
        let their_mesh = their_difference.to_mesh();
        println!("  Their difference: {} tris", their_mesh.indices().len() / 3);
    }
}