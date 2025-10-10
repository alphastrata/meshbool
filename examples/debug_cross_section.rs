use meshbool::{cross_section, cube};
use nalgebra::Vector3;

fn main() {
    println!("Creating cube...");
    let our_cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    
    println!("Computing cross-section at height 0...");
    let our_section = cross_section(&our_cube, 0.0);
    
    println!("Done.");
    println!("Section triangles: {}", our_section.num_tri());
    println!("Section vertices: {}", our_section.num_vert());
}