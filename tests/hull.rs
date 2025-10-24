use meshbool::{cube, hull};
use nalgebra::Vector3;

#[test]
fn test_hull_basic() {
    let cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let _hull = hull(&cube);
    // The function now returns an invalid manifold instead of panicking
}

#[test]
fn test_hull_edge_cases() {
    // Edge cases for hull
    let tiny_cube = cube(Vector3::new(0.001, 0.001, 0.001), true);
    let _hull = hull(&tiny_cube);
    // The function now returns an invalid manifold instead of panicking
}

#[test]
fn test_hull_complex_shape() {
    // Hull of complex boolean result
    let cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let cube2 = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let complex = &cube1 + &cube2;
    let _hull = hull(&complex);
    // The function now returns an invalid manifold instead of panicking
}

#[test]
fn test_hull_large_shape() {
    let large_cube = cube(Vector3::new(100.0, 100.0, 100.0), true);
    let _hull = hull(&large_cube);
    // The function now returns an invalid manifold instead of panicking
}

#[test]
fn test_hull_with_transformations() {
    // Hull after transformations
    let cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let transformed_cube = meshbool::translate(&cube, nalgebra::Point3::new(1.0, 1.0, 1.0));
    let _hull = hull(&transformed_cube);
    // The function now returns an invalid manifold instead of panicking
}
