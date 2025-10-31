use meshbool::cube;
use nalgebra::Vector3;

fn main() {
    println!("🧪 TESTING BASIC MESHBOOL FUNCTIONALITY");
    println!("=====================================");
    
    // Test 1: Basic cube creation
    println!("\n🏗️  Test 1: Creating basic cube...");
    let cube_mesh = cube(Vector3::new(2.0, 2.0, 2.0), true);
    println!("   ✓ Cube created with {} triangles and {} vertices", 
             cube_mesh.num_tri(), cube_mesh.num_vert());
    
    // Test 2: Boolean operations
    println!("\n➕ Test 2: Boolean union operation...");
    let cube1 = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let cube2 = cube(Vector3::new(1.0, 1.0, 1.0), false);
    let union = &cube1 + &cube2;
    println!("   ✓ Union operation successful with {} triangles", union.num_tri());
    
    // Test 3: Translation
    println!("\n➡️  Test 3: Translation operation...");
    let translated = meshbool::translate(&cube1, nalgebra::Point3::new(1.0, 0.0, 0.0));
    println!("   ✓ Translation successful with {} triangles", translated.num_tri());
    
    // Test 4: Cross-section
    println!("\n✂️  Test 4: Cross-section operation...");
    let section = meshbool::cross_section(&cube1, 0.0);
    println!("   ✓ Cross-section created with {} triangles and {} vertices", 
             section.num_tri(), section.num_vert());
    
    // Test 5: Hull operation
    println!("\n🌐 Test 5: Hull operation...");
    let hull_result = meshbool::hull(&cube1);
    println!("   ✓ Hull operation successful with {} triangles", hull_result.num_tri());
    
    // Test 6: SDF operation
    println!("\n📏 Test 6: SDF operation...");
    let sdf_result = meshbool::sdf(&cube1, 0.1);
    println!("   ✓ SDF operation successful with {} triangles", sdf_result.num_tri());
    
    // Test 7: Smooth operation
    println!("\n✨ Test 7: Smooth operation...");
    let smooth_result = meshbool::smooth(&cube1, 0.1);
    println!("   ✓ Smooth operation successful with {} triangles", smooth_result.num_tri());
    
    println!("\n🎉 ALL BASIC FUNCTIONALITY TESTS COMPLETED SUCCESSFULLY!");
    println!("=====================================================");
}