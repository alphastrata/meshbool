use bevy::prelude::*;
use bevy_step_loader::*;

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            StepPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, debug_step_mesh)
        .run();
}

#[derive(Resource)]
struct StepHandle(Handle<StepAsset>);

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let handle: Handle<StepAsset> = asset_server.load("real_parts/multifeature.step");
    commands.insert_resource(StepHandle(handle));
}

fn debug_step_mesh(
    assets: Res<Assets<StepAsset>>,
    handle: Res<StepHandle>,
) {
    if let Some(asset) = assets.get(&handle.0) {
        println!("=== STEP FILE DEBUG ===");
        println!("Vertices: 28688");
        println!("Indices: 82842");
        println!("Triangles: 27614");
        
        // Debug why mesh conversion fails
        println!("\n=== DETAILED MESH ANALYSIS ===");
        
        // Check vertex data
        if let Some(positions) = asset.mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            match positions {
                bevy::render::mesh::VertexAttributeValues::Float32x3(vertices) => {
                    println!("  Found {} vertices (Float32x3)", vertices.len());
                    
                    // Check for NaN or infinite values
                    let mut has_nan = false;
                    let mut has_inf = false;
                    for (i, vertex) in vertices.iter().enumerate().take(10) {
                        for (j, &coord) in vertex.iter().enumerate() {
                            if coord.is_nan() {
                                println!("  🔴 NaN found at vertex {} coordinate {}", i, j);
                                has_nan = true;
                            }
                            if coord.is_infinite() {
                                println!("  🔴 Infinite value found at vertex {} coordinate {}", i, j);
                                has_inf = true;
                            }
                        }
                    }
                    
                    if !has_nan && !has_inf {
                        println!("  ✅ No NaN or infinite values in first 10 vertices");
                    }
                },
                _ => println!("  Unknown vertex format"),
            }
        } else {
            println!("  🔴 No position attribute found!");
        }
        
        // Check indices
        if let Some(indices) = asset.mesh.indices() {
            match indices {
                bevy::render::mesh::Indices::U32(indices_vec) => {
                    println!("  Found {} U32 indices", indices_vec.len());
                    
                    // Check for out-of-bounds indices
                    if let Some(positions) = asset.mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
                        if let bevy::render::mesh::VertexAttributeValues::Float32x3(vertices) = positions {
                            let max_index = *indices_vec.iter().max().unwrap_or(&0);
                            if max_index >= vertices.len() as u32 {
                                println!("  🔴 ERROR: Index {} is out of bounds for {} vertices", max_index, vertices.len());
                            } else {
                                println!("  ✅ Indices are within bounds (max: {}, vertices: {})", max_index, vertices.len());
                            }
                        }
                    }
                },
                bevy::render::mesh::Indices::U16(indices_vec) => {
                    println!("  Found {} U16 indices", indices_vec.len());
                    
                    // Check for out-of-bounds indices
                    if let Some(positions) = asset.mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
                        if let bevy::render::mesh::VertexAttributeValues::Float32x3(vertices) = positions {
                            let max_index = *indices_vec.iter().max().unwrap_or(&0) as u32;
                            if max_index >= vertices.len() as u32 {
                                println!("  🔴 ERROR: Index {} is out of bounds for {} vertices", max_index, vertices.len());
                            } else {
                                println!("  ✅ Indices are within bounds (max: {}, vertices: {})", max_index, vertices.len());
                            }
                        }
                    }
                },
            }
        } else {
            println!("  🔴 No indices found!");
        }
        
        // Try direct manifold conversion with detailed logging
        println!("\n=== DIRECT MANIFOLD CONVERSION ATTEMPT ===");
        if let Some(positions) = asset.mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            if let bevy::render::mesh::VertexAttributeValues::Float32x3(vertices) = positions {
                if let Some(indices) = asset.mesh.indices() {
                    match indices {
                        bevy::render::mesh::Indices::U32(u32_indices) => {
                            println!("  Attempting conversion with {} vertices and {} U32 indices", vertices.len(), u32_indices.len());
                            
                            // Flatten vertices to f32 array as expected by manifold-rs
                            let start_time = std::time::Instant::now();
                            let flattened_vertices: Vec<f32> = vertices.iter()
                                .flat_map(|v| [v[0], v[1], v[2]])
                                .collect();
                            println!("  Flattened to {} float values in {:?}", flattened_vertices.len(), start_time.elapsed());
                            
                            // Try creating manifold-rs mesh directly
                            let mesh_start = std::time::Instant::now();
                            let mesh = manifold_rs::Mesh::new(&flattened_vertices, u32_indices);
                            let mesh_time = mesh_start.elapsed();
                            println!("  Created manifold-rs mesh in {:?}: {} vertices, {} indices", 
                                     mesh_time, mesh.vertices().len() / 3, mesh.indices().len());
                            
                            // Debug the created mesh properties
                            println!("  Mesh properties: {}", mesh.num_props());
                            println!("  Mesh vertices divided by 3: {}", mesh.vertices().len() / 3);
                            
                            let manifold_start = std::time::Instant::now();
                            let manifold = manifold_rs::Manifold::from_mesh(mesh);
                            let manifold_time = manifold_start.elapsed();
                            
                            let result_mesh = manifold.to_mesh();
                            println!("  Created manifold in {:?}: {} vertices, {} indices", 
                                     manifold_time, result_mesh.vertices().len(), result_mesh.indices().len());
                            
                            // Debug why it's 0
                            if result_mesh.vertices().len() == 0 {
                                println!("  🔴 RESULT IS EMPTY - manifold-rs rejected the mesh!");
                                println!("  🔴 Possible reasons:");
                                println!("    - Mesh might not be watertight (holes)");
                                println!("    - Mesh might have incorrect winding order");
                                println!("    - Mesh might be degenerate or self-intersecting");
                                println!("    - Mesh might have inconsistent normals");
                                
                                // Try with a simple cube to see if manifold-rs works at all
                                println!("\n  === SIMPLE CUBE TEST ===");
                                let cube = manifold_rs::Manifold::cube(1.0, 1.0, 1.0);
                                let cube_mesh = cube.to_mesh();
                                println!("    Simple cube: {} vertices, {} indices", 
                                         cube_mesh.vertices().len(), cube_mesh.indices().len());
                            }
                        },
                        bevy::render::mesh::Indices::U16(u16_indices) => {
                            println!("  Converting {} U16 indices to U32", u16_indices.len());
                            let u32_indices: Vec<u32> = u16_indices.iter().map(|&i| i as u32).collect();
                            
                            // Flatten vertices to f32 array as expected by manifold-rs
                            let flattened_vertices: Vec<f32> = vertices.iter()
                                .flat_map(|v| [v[0], v[1], v[2]])
                                .collect();
                            println!("  Flattened to {} float values", flattened_vertices.len());
                            
                            // Try creating manifold-rs mesh directly
                            let mesh = manifold_rs::Mesh::new(&flattened_vertices, &u32_indices);
                            let manifold = manifold_rs::Manifold::from_mesh(mesh);
                            
                            let result_mesh = manifold.to_mesh();
                            println!("  Created manifold: {} vertices, {} indices", 
                                     result_mesh.vertices().len(), result_mesh.indices().len());
                        },
                    }
                }
            }
        }
        
        std::process::exit(0);
    }
}