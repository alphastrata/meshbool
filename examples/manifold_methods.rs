use manifold_rs::Manifold;

fn main() {
    let cube = Manifold::cube(2.0, 2.0, 2.0);
    
    // Check what methods are available by looking at the mesh
    let mesh = cube.to_mesh();
    println!("Manifold cube:");
    println!("  Mesh vertices: {} ({} points)", mesh.vertices().len(), mesh.vertices().len() / 3);
    println!("  Mesh indices: {} ({} triangles)", mesh.indices().len(), mesh.indices().len() / 3);
    
    // Try slice method
    let slice_result = cube.slice(0.0);
    println!("  Slice at height 0: {} polygons", slice_result.size());
    
    if slice_result.size() > 0 {
        let poly = slice_result.get_as_slice(0);
        println!("    First polygon: {} vertices", poly.len() / 2);
    }
    
    // Try other methods
    let hull_result = cube.hull();
    println!("  Hull: {} triangles", hull_result.to_mesh().indices().len() / 3);
    
    let translated_cube = cube.translate(1.0, 1.0, 1.0);
    println!("  Translated cube: {} triangles", translated_cube.to_mesh().indices().len() / 3);
    
    let scaled_cube = cube.scale(2.0, 2.0, 2.0);
    println!("  Scaled cube: {} triangles", scaled_cube.to_mesh().indices().len() / 3);
}