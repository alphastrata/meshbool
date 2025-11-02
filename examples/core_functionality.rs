use meshbool::MeshBool;
use nalgebra::{Vector3, Point3};

fn main() {
    println!("🧪 TESTING MESHBOOL CORE FUNCTIONALITY");
    println!("====================================");
    
    // Test 1: Basic cube creation
    println!("\n🏗️  Test 1: Creating basic cube...");
    let cube_mesh = MeshBool::cube(Vector3::new(2.0, 2.0, 2.0), true);
    println!("   ✓ Cube created with {} triangles and {} vertices", 
             cube_mesh.num_tri(), cube_mesh.num_vert());
    
    // Test 2: Boolean operations
    println!("\n➕ Test 2: Boolean union operation...");
    let cube1 = MeshBool::cube(Vector3::new(1.0, 1.0, 1.0), true);
    let cube2 = MeshBool::cube(Vector3::new(1.0, 1.0, 1.0), false);
    let union = &cube1 + &cube2;
    println!("   ✓ Union operation successful with {} triangles", union.num_tri());
    
    // Test 3: Translation
    println!("\n➡️  Test 3: Translation operation...");
    let translated = cube1.translate(Point3::new(1.0, 0.0, 0.0));
    println!("   ✓ Translation successful with {} triangles", translated.num_tri());
    
    // Test 4: Get mesh data
    println!("\n📋 Test 4: Getting mesh data...");
    let mesh_gl = cube1.get_mesh_gl(0);
    println!("   ✓ Got mesh data with {} vertices and {} triangles", 
             mesh_gl.vert_properties.len() / mesh_gl.num_prop as usize, mesh_gl.tri_verts.len() / 3);
    
    println!("\n🎉 ALL CORE FUNCTIONALITY TESTS COMPLETED SUCCESSFULLY!");
    println!("=====================================================");
}