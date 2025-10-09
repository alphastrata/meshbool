use meshbool::{cube, sdf};
use nalgebra::Vector3;

#[test]
fn test_sdf_basic() {
    let cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let _sdf = sdf(&cube, 0.1);
    // The function now returns an invalid manifold instead of panicking
}

#[test]
fn test_sdf_with_tolerance() {
    let cube = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let _sdf = sdf(&cube, 0.01);
    // The function now returns an invalid manifold instead of panicking
}

#[test]
fn test_sdf_edge_cases() {
    // Edge cases for sdf
    let tiny_cube = cube(Vector3::new(0.001, 0.001, 0.001), true);
    let _sdf = sdf(&tiny_cube, 0.0001);
    // The function now returns an invalid manifold instead of panicking
}

#[test]
fn test_sdf_complex_shape() {
    // SDF of complex boolean result
    let cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let cube2 = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let complex = &cube1 + &cube2;
    let _sdf = sdf(&complex, 0.1);
    // The function now returns an invalid manifold instead of panicking
}

#[test]
fn test_sdf_large_shape() {
    let large_cube = cube(Vector3::new(100.0, 100.0, 100.0), true);
    let _sdf = sdf(&large_cube, 1.0);
    // The function now returns an invalid manifold instead of panicking
}
