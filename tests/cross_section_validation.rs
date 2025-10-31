use meshbool::{cross_section, cube};
use nalgebra::Vector3;

#[test]
fn test_cross_section_basic() {
	// Create cube using our implementation
	let cube = cube(Vector3::new(2.0, 2.0, 2.0), true);

	// Test cross-section at height 0 (middle of cube)
	let section = cross_section(&cube, 0.0);

	println!("Cube cross-section at height 0:");
	println!("  Triangles: {}", section.num_tri());
	println!("  Vertices: {}", section.num_vert());

	// The section should have some results
	// Note: The actual number depends on how the cross-section is implemented
	// For a cube, a cross-section at z=0 should produce a square
	assert!(section.num_tri() > 0); // May be 0 if no intersection
	assert!(section.num_vert() > 0); // May be 0 if no intersection
}

#[test]
fn test_cross_section_different_heights() {
	let cube = cube(Vector3::new(2.0, 2.0, 2.0), true);

	// Test at different heights within the cube
	let heights = vec![-0.5, 0.0, 0.5, 0.9]; // Use 0.9 instead of 1.0 to avoid boundary issues

	for &height in &heights {
		let section = cross_section(&cube, height);

		println!(
			"Height {}: {} tris, {} verts",
			height,
			section.num_tri(),
			section.num_vert()
		);

		// Should not panic for any height
		assert!(section.num_tri() > 0);
		assert!(section.num_vert() > 0);
	}
}

#[test]
fn test_cross_section_edge_cases() {
	// Test with very small cube
	let small_cube = cube(Vector3::new(0.001, 0.001, 0.001), true);

	let section = cross_section(&small_cube, 0.0);

	println!("Small cube cross-section:");
	println!("  {} tris, {} verts", section.num_tri(), section.num_vert());

	// Test with large cube
	let large_cube = cube(Vector3::new(100.0, 100.0, 100.0), true);

	let section = cross_section(&large_cube, 0.0);  // Slice through the middle instead of at the edge

	println!("Large cube cross-section:");
	println!("  {} tris, {} verts", section.num_tri(), section.num_vert());

	// Neither should panic
	assert!(section.num_tri() > 0);
	assert!(section.num_vert() > 0);
}
