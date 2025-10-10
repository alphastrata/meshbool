use manifold_rs::Manifold;

fn main() {
    // Create a simple cube centered at origin (2x2x2)
    let cube = Manifold::cube(2.0, 2.0, 2.0);
    
    // Slice it at height 0 (middle of the cube)
    let slice = cube.slice(0.0);
    
    println!("Slice has {} polygons", slice.size());
    
    // Examine the first polygon
    if slice.size() > 0 {
        let polygon = slice.get_as_slice(0);
        let num_vertices = polygon.len() / 2;
        println!("First polygon has {} vertices", num_vertices);
        
        // Print all vertices
        println!("All vertices:");
        for i in 0..num_vertices {
            println!("  [{:.2}, {:.2}]", polygon[i*2], polygon[i*2+1]);
        }
    }
    
    // Try slicing at different heights
    println!("\n--- Slicing at different heights ---");
    for height in [-1.0, -0.5, 0.0, 0.5, 1.0].iter() {
        let slice_at_height = cube.slice(*height);
        println!("At height {}: {} polygons", height, slice_at_height.size());
        if slice_at_height.size() > 0 {
            let polygon = slice_at_height.get_as_slice(0);
            println!("  First polygon has {} vertices", polygon.len() / 2);
        }
    }
}