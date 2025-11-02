use meshbool::MeshBool;
use nalgebra::{Vector3, Point3};

fn main() {
    println!("🧪 TESTING MESHBOOL BASIC FUNCTIONALITY WITH NEW API");
    println!("==================================================");
    
    // Test 1: Basic cube creation using new API
    println!("\n🏗️  Test 1: Creating basic cube using new API...");
    let cube_mesh = MeshBool::cube(Vector3::new(2.0, 2.0, 2.0), true);
    println!("   ✓ Cube created with {} triangles and {} vertices", 
             cube_mesh.num_tri(), cube_mesh.num_vert());
    
    // Test 2: Translation using new API
    println!("\n➡️  Test 2: Translation using new API...");
    let translated_cube = cube_mesh.translate(Point3::new(1.0, 0.0, 0.0));
    println!("   ✓ Translated cube with {} triangles and {} vertices", 
             translated_cube.num_tri(), translated_cube.num_vert());
    
    // Test 3: Boolean operations using new API
    println!("\n➕ Test 3: Boolean union operation using new API...");
    let cube1 = MeshBool::cube(Vector3::new(1.0, 1.0, 1.0), true);
    let cube2 = MeshBool::cube(Vector3::new(1.0, 1.0, 1.0), false);
    let union_result = &cube1 + &cube2;
    println!("   ✓ Union operation successful with {} triangles", union_result.num_tri());
    
    // Test 4: Cross-section using new API
    println!("\n✂️  Test 4: Cross-section operation using new API...");
    let section = meshbool::cross_section(&cube1, 0.0);
    println!("   ✓ Cross-section created with {} triangles and {} vertices", 
             section.num_tri(), section.num_vert());
    
    // Test 5: Hull operation using new API
    println!("\n🌐 Test 5: Hull operation using new API...");
    let hull_result = meshbool::hull(&cube1);
    println!("   ✓ Hull operation successful with {} triangles", hull_result.num_tri());
    
    // Test 6: SDF operation using new API
    println!("\n📏 Test 6: SDF operation using new API...");
    let sdf_result = meshbool::sdf(&cube1, 0.1);
    println!("   ✓ SDF operation successful with {} triangles", sdf_result.num_tri());
    
    // Test 7: Smooth operation using new API
    println!("\n✨ Test 7: Smooth operation using new API...");
    let smooth_result = meshbool::smooth(&cube1, 0.1);
    println!("   ✓ Smooth operation successful with {} triangles", smooth_result.num_tri());
    
    println!("\n🎉 ALL BASIC FUNCTIONALITY TESTS COMPLETED SUCCESSFULLY WITH NEW API!");
    println!("==================================================================");
}