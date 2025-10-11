use manifold_rs::Manifold;

fn main() {
    let cube = Manifold::cube(2.0, 2.0, 2.0);
    
    // Check what methods are available on Manifold
    println!("Manifold methods:");
    
    // Try cross-section (slice)
    let slice_result = cube.slice(0.0);
    println!("Slice result: {} polygons", slice_result.size());
    
    // Try hull
    let hull_result = cube.hull();
    println!("Hull result triangles: {}", hull_result.to_mesh().indices().len() / 3);
    
    // Try to_mesh method
    let mesh = cube.to_mesh();
    println!("Mesh: {} vertices, {} indices ({} triangles)", 
             mesh.vertices().len() / 3, 
             mesh.indices().len(), 
             mesh.indices().len() / 3);
    
    // Try other methods
    let translated = cube.translate(1.0, 1.0, 1.0);
    println!("Translated triangles: {}", translated.to_mesh().indices().len() / 3);
    
    let scaled = cube.scale(2.0, 2.0, 2.0);
    println!("Scaled triangles: {}", scaled.to_mesh().indices().len() / 3);
}