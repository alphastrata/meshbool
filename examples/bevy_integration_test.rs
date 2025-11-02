#[cfg(feature = "bevy")]
use meshbool::{MeshBool, meshbool_to_bevy_mesh, bevy_mesh_to_meshbool};
#[cfg(feature = "bevy")]
use nalgebra::Vector3;

#[cfg(feature = "bevy")]
fn main() {
    println!("🧪 TESTING BEVY INTEGRATION");
    println!("=========================");
    
    // Test 1: Create a MeshBool cube
    println!("\n🏗️  Test 1: Creating MeshBool cube...");
    let cube_mesh = MeshBool::cube(Vector3::new(2.0, 2.0, 2.0), true);
    println!("   ✓ Cube created with {} triangles and {} vertices", 
             cube_mesh.num_tri(), cube_mesh.num_vert());
    
    // Test 2: Convert to Bevy mesh
    println!("\n🔄 Test 2: Converting MeshBool to Bevy mesh...");
    let bevy_mesh = meshbool_to_bevy_mesh(&cube_mesh);
    println!("   ✓ Converted to Bevy mesh");
    
    // Test 3: Convert back to MeshBool
    println!("\n🔄 Test 3: Converting Bevy mesh back to MeshBool...");
    let converted_mesh = bevy_mesh_to_meshbool(&bevy_mesh);
    if let Some(mesh) = converted_mesh {
        println!("   ✓ Converted back to MeshBool with {} triangles and {} vertices", 
                 mesh.num_tri(), mesh.num_vert());
    } else {
        println!("   ⚠️  Conversion failed");
    }
    
    // Test 4: Round-trip conversion
    println!("\n🔁 Test 4: Round-trip conversion...");
    let roundtrip_mesh = meshbool_to_bevy_mesh(&cube_mesh);
    let roundtrip_converted = bevy_mesh_to_meshbool(&roundtrip_mesh);
    if let Some(mesh) = roundtrip_converted {
        println!("   ✓ Round-trip successful with {} triangles and {} vertices", 
                 mesh.num_tri(), mesh.num_vert());
    } else {
        println!("   ⚠️  Round-trip conversion failed");
    }
    
    println!("\n🎉 BEVY INTEGRATION TESTS COMPLETED!");
    println!("==================================");
}

#[cfg(not(feature = "bevy"))]
fn main() {
    println!("This example requires the 'bevy' feature to be enabled.");
    println!("Run with: cargo run --example bevy_integration --features bevy");
}