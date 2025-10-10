use manifold_rs::Manifold;

fn main() {
    let cube = Manifold::cube(2.0, 2.0, 2.0);
    println!("Manifold created successfully");
    
    let mesh = cube.to_mesh();
    println!("Mesh converted successfully");
    
    let vertices = mesh.vertices();
    let indices = mesh.indices();
    
    println!("Mesh statistics:");
    println!("  Vertices: {} ({} points)", vertices.len(), vertices.len() / 3);
    println!("  Indices: {} ({} triangles)", indices.len(), indices.len() / 3);
    
    // Try to get some mesh statistics
    println!("Vertices (first 3):");
    for i in (0..vertices.len().min(9)).step_by(3) {
        println!("  [{:.2}, {:.2}, {:.2}]", vertices[i], vertices[i+1], vertices[i+2]);
    }
    
    println!("Indices (first 3 triangles):");
    for i in (0..indices.len().min(9)).step_by(3) {
        println!("  [{}, {}, {}]", indices[i], indices[i+1], indices[i+2]);
    }
    
    // Try some boolean operations
    let cube2 = Manifold::cube(1.0, 1.0, 1.0);
    let union = cube.union(&cube2);
    let intersection = cube.intersection(&cube2);
    let difference = cube.difference(&cube2);
    
    println!("Boolean operations:");
    let union_mesh = union.to_mesh();
    let intersection_mesh = intersection.to_mesh();
    let difference_mesh = difference.to_mesh();
    
    println!("  Union: {} vertices, {} triangles", 
             union_mesh.vertices().len() / 3, union_mesh.indices().len() / 3);
    println!("  Intersection: {} vertices, {} triangles", 
             intersection_mesh.vertices().len() / 3, intersection_mesh.indices().len() / 3);
    println!("  Difference: {} vertices, {} triangles", 
             difference_mesh.vertices().len() / 3, difference_mesh.indices().len() / 3);
}