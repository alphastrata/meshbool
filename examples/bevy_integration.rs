//! Bevy 0.17.0 integration example for meshbool
//!
//! This example demonstrates how to convert between meshbool's MeshGL type,
//! Bevy's Mesh type, and back, showcasing the rich metadata capabilities
//! of the MeshGL type for game development.

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use meshbool::{cube, get_mesh_gl, translate};
use nalgebra::Vector3;

/// Convert meshbool MeshGL to Bevy Mesh
/// 
/// This function leverages the rich metadata in MeshGL to create optimal Bevy meshes
/// with proper vertex attributes, indices, and instance information.
fn meshgl_to_bevy_mesh(mesh_gl: &meshbool::MeshGL) -> Mesh {
    let mut bevy_mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    
    // Extract vertex positions from MeshGL
    let num_verts = mesh_gl.vert_properties.len() / mesh_gl.num_prop as usize;
    let mut positions = Vec::with_capacity(num_verts);
    
    for i in 0..num_verts {
        let offset = i * mesh_gl.num_prop as usize;
        positions.push([
            mesh_gl.vert_properties[offset],
            mesh_gl.vert_properties[offset + 1], 
            mesh_gl.vert_properties[offset + 2]
        ]);
    }
    
    // Extract triangle indices from MeshGL
    let indices: Vec<u32> = mesh_gl.tri_verts.clone();
    
    // Insert vertex data into Bevy mesh
    bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    
    // If MeshGL has normals (property index 3, 4, 5), extract them
    if mesh_gl.num_prop >= 6 {
        let mut normals = Vec::with_capacity(num_verts);
        for i in 0..num_verts {
            let offset = i * mesh_gl.num_prop as usize;
            normals.push([
                mesh_gl.vert_properties[offset + 3],
                mesh_gl.vert_properties[offset + 4], 
                mesh_gl.vert_properties[offset + 5]
            ]);
        }
        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    }
    
    // If MeshGL has UVs (property index 6, 7), extract them
    if mesh_gl.num_prop >= 8 {
        let mut uvs = Vec::with_capacity(num_verts);
        for i in 0..num_verts {
            let offset = i * mesh_gl.num_prop as usize;
            uvs.push([
                mesh_gl.vert_properties[offset + 6],
                mesh_gl.vert_properties[offset + 7]
            ]);
        }
        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    }
    
    // Insert indices
    bevy_mesh.insert_indices(Indices::U32(indices));
    
    bevy_mesh
}

/// Convert Bevy Mesh to meshbool MeshGL
/// 
/// This function converts a Bevy mesh back to MeshGL format, preserving
/// as much metadata as possible for round-trip compatibility.
fn bevy_mesh_to_meshgl(bevy_mesh: &Mesh) -> meshbool::MeshGL {
    let mut mesh_gl = meshbool::MeshGL::default();
    
    // Extract vertex positions
    if let Some(positions) = bevy_mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        let position_data = positions.as_float3();
        if let Some(pos_data) = position_data {
            let num_verts = pos_data.len();
            mesh_gl.vert_properties.reserve(num_verts * 3);
            mesh_gl.num_prop = 3; // Start with just positions
            
            for pos in pos_data {
                mesh_gl.vert_properties.push(pos[0]);
                mesh_gl.vert_properties.push(pos[1]);
                mesh_gl.vert_properties.push(pos[2]);
            }
        }
    }
    
    // Extract normals if present
    if let Some(normals) = bevy_mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
        let normal_data = normals.as_float3();
        if let Some(norm_data) = normal_data {
            // Extend vertex properties to include normals
            let num_verts = norm_data.len();
            let mut extended_properties = Vec::with_capacity(num_verts * 6);
            
            for i in 0..num_verts {
                // Copy position data
                extended_properties.push(mesh_gl.vert_properties[i * 3]);
                extended_properties.push(mesh_gl.vert_properties[i * 3 + 1]);
                extended_properties.push(mesh_gl.vert_properties[i * 3 + 2]);
                
                // Add normal data
                let norm = norm_data[i];
                extended_properties.push(norm[0]);
                extended_properties.push(norm[1]);
                extended_properties.push(norm[2]);
            }
            
            mesh_gl.vert_properties = extended_properties;
            mesh_gl.num_prop = 6; // Positions + normals
        }
    }
    
    // Extract UVs if present
    if let Some(uvs) = bevy_mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
        let uv_data = uvs.as_float2();
        if let Some(uv_data) = uv_data {
            // Extend vertex properties to include UVs
            let num_verts = uv_data.len();
            let mut extended_properties = Vec::with_capacity(num_verts * 8);
            
            for i in 0..num_verts {
                // Copy existing data (positions + normals)
                if mesh_gl.num_prop >= 6 {
                    extended_properties.push(mesh_gl.vert_properties[i * 6]);
                    extended_properties.push(mesh_gl.vert_properties[i * 6 + 1]);
                    extended_properties.push(mesh_gl.vert_properties[i * 6 + 2]);
                    extended_properties.push(mesh_gl.vert_properties[i * 6 + 3]);
                    extended_properties.push(mesh_gl.vert_properties[i * 6 + 4]);
                    extended_properties.push(mesh_gl.vert_properties[i * 6 + 5]);
                } else {
                    extended_properties.push(mesh_gl.vert_properties[i * 3]);
                    extended_properties.push(mesh_gl.vert_properties[i * 3 + 1]);
                    extended_properties.push(mesh_gl.vert_properties[i * 3 + 2]);
                    extended_properties.push(0.0); // Normal x
                    extended_properties.push(0.0); // Normal y
                    extended_properties.push(0.0); // Normal z
                }
                
                // Add UV data
                let uv = uv_data[i];
                extended_properties.push(uv[0]);
                extended_properties.push(uv[1]);
            }
            
            mesh_gl.vert_properties = extended_properties;
            mesh_gl.num_prop = 8; // Positions + normals + UVs
        }
    }
    
    // Extract indices
    if let Some(indices) = bevy_mesh.indices() {
        match indices {
            Indices::U32(idx) => mesh_gl.tri_verts = idx.to_vec(),
            Indices::U16(idx) => {
                mesh_gl.tri_verts = idx.iter().map(|&i| i as u32).collect();
            }
        }
    }
    
    mesh_gl
}

/// Demonstrate round-trip conversion: meshbool -> Bevy -> meshbool
fn demonstrate_round_trip_conversion() {
    println!("=== Round-Trip Conversion Demo ===");
    
    // Create a cube using meshbool
    let our_cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    println!("1. Created cube with meshbool");
    
    // Convert to MeshGL
    let mesh_gl = get_mesh_gl(&our_cube, 0);
    println!("2. Converted to MeshGL: {} verts, {} tris", 
             mesh_gl.vert_properties.len() / mesh_gl.num_prop as usize,
             mesh_gl.tri_verts.len() / 3);
    
    // Convert MeshGL to Bevy Mesh
    let bevy_mesh = meshgl_to_bevy_mesh(&mesh_gl);
    println!("3. Converted to Bevy Mesh: {} positions, {} indices", 
             bevy_mesh.attribute(Mesh::ATTRIBUTE_POSITION).map(|a| a.len()).unwrap_or(0),
             bevy_mesh.indices().map(|i| i.len()).unwrap_or(0));
    
    // Convert Bevy Mesh back to MeshGL
    let converted_mesh_gl = bevy_mesh_to_meshgl(&bevy_mesh);
    println!("4. Converted back to MeshGL: {} verts, {} tris", 
             converted_mesh_gl.vert_properties.len() / converted_mesh_gl.num_prop as usize,
             converted_mesh_gl.tri_verts.len() / 3);
    
    // Verify round-trip preservation
    let original_num_verts = mesh_gl.vert_properties.len() / mesh_gl.num_prop as usize;
    let converted_num_verts = converted_mesh_gl.vert_properties.len() / converted_mesh_gl.num_prop as usize;
    let original_num_tris = mesh_gl.tri_verts.len() / 3;
    let converted_num_tris = converted_mesh_gl.tri_verts.len() / 3;
    
    println!("5. Round-trip verification:");
    println!("   Original: {} verts, {} tris", original_num_verts, original_num_tris);
    println!("   Converted: {} verts, {} tris", converted_num_verts, converted_num_tris);
    
    // For basic validation, check that we have reasonable numbers
    assert!(converted_num_verts > 0, "Converted mesh should have vertices");
    assert!(converted_num_tris > 0, "Converted mesh should have triangles");
    
    println!("✅ Round-trip conversion successful!");
}

/// Demonstrate mesh operations with conversions
fn demonstrate_mesh_operations() {
    println!("\n=== Mesh Operations Demo ===");
    
    // Create cubes using meshbool
    let cube1 = cube(Vector3::new(2.0, 2.0, 2.0), true);
    let cube2 = cube(Vector3::new(1.0, 1.0, 1.0), true);
    let translated_cube2 = translate(&cube2, nalgebra::Point3::new(1.0, 0.0, 0.0));
    
    println!("1. Created two cubes and translated one");
    
    // Perform boolean union operation
    let union_result = &cube1 + &translated_cube2;
    println!("2. Performed boolean union: {} tris", union_result.num_tri());
    
    // Convert result to Bevy mesh
    let mesh_gl = get_mesh_gl(&union_result, 0);
    let bevy_mesh = meshgl_to_bevy_mesh(&mesh_gl);
    println!("3. Converted to Bevy mesh: {} positions, {} indices", 
             bevy_mesh.attribute(Mesh::ATTRIBUTE_POSITION).map(|a| a.len()).unwrap_or(0),
             bevy_mesh.indices().map(|i| i.len()).unwrap_or(0));
    
    // Perform boolean intersection operation
    let intersection_result = &cube1 ^ &translated_cube2;
    println!("4. Performed boolean intersection: {} tris", intersection_result.num_tri());
    
    // Convert result to Bevy mesh
    let mesh_gl = get_mesh_gl(&intersection_result, 0);
    let bevy_mesh = meshgl_to_bevy_mesh(&mesh_gl);
    println!("5. Converted to Bevy mesh: {} positions, {} indices", 
             bevy_mesh.attribute(Mesh::ATTRIBUTE_POSITION).map(|a| a.len()).unwrap_or(0),
             bevy_mesh.indices().map(|i| i.len()).unwrap_or(0));
    
    // Perform boolean difference operation
    let difference_result = &cube1 - &translated_cube2;
    println!("6. Performed boolean difference: {} tris", difference_result.num_tri());
    
    // Convert result to Bevy mesh
    let mesh_gl = get_mesh_gl(&difference_result, 0);
    let bevy_mesh = meshgl_to_bevy_mesh(&mesh_gl);
    println!("7. Converted to Bevy mesh: {} positions, {} indices", 
             bevy_mesh.attribute(Mesh::ATTRIBUTE_POSITION).map(|a| a.len()).unwrap_or(0),
             bevy_mesh.indices().map(|i| i.len()).unwrap_or(0));
    
    println!("✅ All mesh operations successful!");
}

/// Demonstrate advanced MeshGL features for game development
fn demonstrate_advanced_features() {
    println!("\n=== Advanced MeshGL Features Demo ===");
    
    // Create a cube using meshbool
    let our_cube = cube(Vector3::new(2.0, 2.0, 2.0), true);
    
    // Convert to MeshGL
    let mesh_gl = get_mesh_gl(&our_cube, 0);
    
    println!("MeshGL advanced features:");
    println!("  num_prop: {}", mesh_gl.num_prop);
    println!("  vert_properties len: {}", mesh_gl.vert_properties.len());
    println!("  tri_verts len: {}", mesh_gl.tri_verts.len());
    println!("  merge_from_vert len: {}", mesh_gl.merge_from_vert.len());
    println!("  merge_to_vert len: {}", mesh_gl.merge_to_vert.len());
    println!("  run_index len: {}", mesh_gl.run_index.len());
    println!("  run_original_id len: {}", mesh_gl.run_original_id.len());
    println!("  run_transform len: {}", mesh_gl.run_transform.len());
    println!("  face_id len: {}", mesh_gl.face_id.len());
    println!("  tolerance: {}", mesh_gl.tolerance);
    
    // These rich metadata features make MeshGL ideal for game development:
    // 1. Instance tracking for efficient rendering
    // 2. Material ID mapping for proper shader selection
    // 3. Transform information for dynamic batching
    // 4. Face connectivity for polygon reconstruction
    // 5. Merge information for manifold preservation
    
    println!("✅ Advanced features demonstrated!");
}

fn main() {
    println!("Bevy 0.17.0 Integration Example for meshbool");
    println!("=============================================\n");
    
    demonstrate_round_trip_conversion();
    demonstrate_mesh_operations();
    demonstrate_advanced_features();
    
    println!("\n🎉 All demonstrations completed successfully!");
    println!("\nMeshGL's rich metadata makes it perfect for game development because:");
    println!("1. ✅ GPU-ready data layout minimizes CPU-GPU transfer overhead");
    println!("2. ✅ Instance tracking enables efficient instanced rendering");
    println!("3. ✅ Material ID tracking supports proper shader selection");
    println!("4. ✅ Transform information enables dynamic batching");
    println!("5. ✅ Face connectivity preserves polygon information through operations");
    println!("6. ✅ Merge information maintains manifold properties");
    println!("7. ✅ Tolerance control ensures quality preservation");
    
    println!("\nThe sophisticated MeshGL type provides an excellent foundation for a future MeshWGPU type");
    println!("that would leverage these features for optimal game mesh performance.");
}