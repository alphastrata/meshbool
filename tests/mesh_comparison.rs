use meshbool::{cube, approx_eq, translate};
use nalgebra::Vector3;

#[test]
fn test_cube_approx_eq() {
    let our_cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let their_cube = manifold_rs::Manifold::cube(2.0, 2.0, 2.0);
    
    let result = approx_eq!(&our_cube, &their_cube);
    assert!(result, "Basic cube mesh should be approximately equal");
}

#[test]
fn test_translated_cube_approx_eq() {
    let our_cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let translated_our_cube = translate(&our_cube, nalgebra::Point3::new(1.0, 1.0, 1.0));
    
    let their_cube = manifold_rs::Manifold::cube(2.0, 2.0, 2.0);
    let translated_their_cube = their_cube.translate(1.0, 1.0, 1.0);
    
    let result = approx_eq!(&translated_our_cube, &translated_their_cube);
    assert!(result, "Translated cube meshes should be approximately equal");
}

#[test]
fn test_boolean_union_approx_eq() {
    let our_cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let our_cube2 = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let our_union = &our_cube1 + &our_cube2;
    
    let their_cube1 = manifold_rs::Manifold::cube(2.0, 2.0, 2.0);
    let their_cube2 = manifold_rs::Manifold::cube(1.0, 1.0, 1.0);
    let their_union = &their_cube1 + &their_cube2;
    
    let result = approx_eq!(&our_union, &their_union);
    assert!(result, "Union of cubes should be approximately equal");
}

#[test]
fn test_boolean_intersection_approx_eq() {
    let our_cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let our_cube2 = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let our_intersection = &our_cube1 ^ &our_cube2;
    
    let their_cube1 = manifold_rs::Manifold::cube(2.0, 2.0, 2.0);
    let their_cube2 = manifold_rs::Manifold::cube(1.0, 1.0, 1.0);
    let their_intersection = &their_cube1 ^ &their_cube2;
    
    let result = approx_eq!(&our_intersection, &their_intersection);
    assert!(result, "Intersection of cubes should be approximately equal");
}

#[test]
fn test_boolean_difference_approx_eq() {
    let our_cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let our_cube2 = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let our_difference = &our_cube1 - &our_cube2;
    
    let their_cube1 = manifold_rs::Manifold::cube(2.0, 2.0, 2.0);
    let their_cube2 = manifold_rs::Manifold::cube(1.0, 1.0, 1.0);
    let their_difference = &their_cube1 - &their_cube2;
    
    let result = approx_eq!(&our_difference, &their_difference);
    assert!(result, "Difference of cubes should be approximately equal");
}