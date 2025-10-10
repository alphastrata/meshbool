use manifold_rs::Manifold;

fn main() {
    let cube = Manifold::cube(2.0, 2.0, 2.0);
    let mesh = cube.to_mesh();
    
    println!("Manifold cube:");
    println!("  Vertices: {} ({} points)", mesh.vertices().len(), mesh.vertices().len() / 3);
    println!("  Indices: {} ({} triangles)", mesh.indices().len(), mesh.indices().len() / 3);
    
    // Try to find bounding box methods
    println!("Bounding box methods:");
    // Try different method names for bounding box
    // let bbox = cube.bounding_box();
    // let bbox = cube.bbox();
    // let bbox = cube.get_bounding_box();
    
    println!("Methods available on Manifold:");
    // This will show what methods are available when we compile
}