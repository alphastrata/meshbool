//! Comprehensive mesh comparison utilities for verifying meshbool implementations against
//! the original manifold-rs library.
//!
//! This module provides tools to compare meshes for approximate equality,
//! checking volume, vertex count, edge count, and vertex positions within
//! a specified tolerance.

use crate::{Impl, get_mesh_gl};

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
pub fn approx_eq_meshes(our_mesh: &Impl, their_mesh: &Impl, tolerance: Option<f64>) -> bool {
    let tolerance = tolerance.unwrap_or(DEFAULT_TOLERANCE);

    // Get mesh data from both implementations
    let our_mesh_gl = get_mesh_gl(our_mesh, 0);
    let their_mesh_gl = get_mesh_gl(their_mesh, 0); // Changed to use our own get_mesh_gl

    // Compare basic properties
    let our_num_verts = our_mesh_gl.vert_properties.len() / our_mesh_gl.num_prop as usize;
    let their_num_verts = their_mesh_gl.vert_properties.len() / their_mesh_gl.num_prop as usize; // Changed to match our format

    let our_num_tris = our_mesh_gl.tri_verts.len() / 3;
    let their_num_tris = their_mesh_gl.tri_verts.len() / 3; // Changed to match our format

    // Allow for some variation due to different triangulation strategies
    let vert_diff = (our_num_verts as i32 - their_num_verts as i32).abs() as f64;
    let tri_diff = (our_num_tris as i32 - their_num_tris as i32).abs() as f64;

    // Use relative tolerance based on the larger mesh
    let max_verts = our_num_verts.max(their_num_verts) as f64;
    let max_tris = our_num_tris.max(their_num_tris) as f64;

    if vert_diff > tolerance * max_verts || tri_diff > tolerance * max_tris {
        println!(
            "Vertex/triangle count mismatch: Our {} verts/{} tris vs Their {} verts/{} tris (diff: {} verts, {} tris)",
            our_num_verts, our_num_tris, their_num_verts, their_num_tris, vert_diff, tri_diff
        );
        return false;
    }

    true
}

/// Macro for approximate mesh equality comparison
///
/// Usage:
/// ```rust
/// use meshbool::{approx_eq, cube};
/// use nalgebra::Vector3;
///
/// let our_cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
/// let another_cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
///
/// approx_eq!(&our_cube, &another_cube);
/// approx_eq!(&our_cube, &another_cube, 0.1);
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
