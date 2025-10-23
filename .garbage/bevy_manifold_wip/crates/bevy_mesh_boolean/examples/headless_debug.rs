use bevy::prelude::*;
use bevy_mesh_boolean::*;
use bevy_step_loader::*;
use std::process;

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            StepPlugin,
            MeshBooleanPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, debug_mesh_conversion)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Load the STEP file
    let step_handle: Handle<StepAsset> = asset_server.load("real_parts/multifeature.step");
    commands.insert_resource(StepHandleResource(step_handle));
    
    // Add a simple resource to track our debugging state
    commands.insert_resource(DebugState::default());
}

#[derive(Resource)]
struct StepHandleResource(Handle<StepAsset>);

#[derive(Resource, Default)]
struct DebugState {
    step_processed: bool,
}

fn debug_mesh_conversion(
    mut commands: Commands,
    step_assets: Res<Assets<StepAsset>>,
    step_handle_resource: Res<StepHandleResource>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut debug_state: ResMut<DebugState>,
) {
    if debug_state.step_processed {
        return;
    }
    
    if let Some(step_asset) = step_assets.get(&step_handle_resource.0) {
        eprintln!("=== STEP FILE DEBUG INFO ===");
        
        // Get mesh statistics
        let vertex_count = if let Some(positions) = step_asset.mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            match positions {
                bevy::render::mesh::VertexAttributeValues::Float32x3(pos) => pos.len(),
                _ => 0,
            }
        } else {
            0
        };
        
        let index_count = if let Some(indices) = step_asset.mesh.indices() {
            match indices {
                bevy::render::mesh::Indices::U32(indices_vec) => indices_vec.len(),
                bevy::render::mesh::Indices::U16(indices_vec) => indices_vec.len(),
                _ => 0,
            }
        } else {
            0
        };
        
        let triangle_count = index_count / 3;
        
        eprintln!("STEP mesh statistics:");
        eprintln!("  Vertices: {}", vertex_count);
        eprintln!("  Indices: {}", index_count);
        eprintln!("  Triangles: {}", triangle_count);
        
        // Try to convert to manifold
        eprintln!("\n=== MESH TO MANIFOLD CONVERSION ATTEMPT ===");
        let conversion_result = bevy_mesh_to_manifold(&step_asset.mesh);
        
        match conversion_result {
            Some(manifold) => {
                eprintln!("✅ SUCCESS: Mesh converted to manifold!");
                let mesh_info = manifold.to_mesh();
                eprintln!("  Manifold vertices: {}", mesh_info.vertices().len());
                eprintln!("  Manifold indices: {}", mesh_info.indices().len());
                eprintln!("  Manifold triangles: {}", mesh_info.indices().len() / 3);
                eprintln!("  Manifold properties: {}", mesh_info.num_props());
                
                // Try converting back to Bevy mesh
                eprintln!("\n=== MANIFOLD TO BEVY MESH CONVERSION ===");
                let bevy_mesh = manifold_to_bevy_mesh(manifold);
                
                let result_vertex_count = if let Some(positions) = bevy_mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
                    match positions {
                        bevy::render::mesh::VertexAttributeValues::Float32x3(pos) => pos.len(),
                        _ => 0,
                    }
                } else {
                    0
                };
                
                let result_index_count = if let Some(indices) = bevy_mesh.indices() {
                    match indices {
                        bevy::render::mesh::Indices::U32(indices_vec) => indices_vec.len(),
                        bevy::render::mesh::Indices::U16(indices_vec) => indices_vec.len(),
                        _ => 0,
                    }
                } else {
                    0
                };
                
                eprintln!("Resulting Bevy mesh:");
                eprintln!("  Vertices: {}", result_vertex_count);
                eprintln!("  Indices: {}", result_index_count);
                eprintln!("  Triangles: {}", result_index_count / 3);
                
                // Try a simple boolean operation with a cube
                eprintln!("\n=== BOOLEAN OPERATION TEST ===");
                let cube = manifold_rs::Manifold::cube(1.0, 1.0, 1.0);
                let translated_cube = cube.translate(0.5, 0.0, 0.0);
                let result = manifold.boolean_op(&translated_cube, manifold_rs::BooleanOp::Difference);
                
                let result_mesh = result.to_mesh();
                eprintln!("Boolean operation result:");
                eprintln!("  Vertices: {}", result_mesh.vertices().len());
                eprintln!("  Indices: {}", result_mesh.indices().len());
                eprintln!("  Triangles: {}", result_mesh.indices().len() / 3);
            },
            None => {
                eprintln!("❌ FAILED: Could not convert mesh to manifold");
                
                // Debug the mesh in detail
                eprintln!("\n=== DETAILED MESH ANALYSIS ===");
                
                // Check vertex attributes
                if let Some(attribute) = step_asset.mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
                    match attribute {
                        bevy::render::mesh::VertexAttributeValues::Float32x3(vertices) => {
                            eprintln!("  Position attribute: {} vertices", vertices.len());
                            if !vertices.is_empty() {
                                eprintln!("  First vertex: {:?}", vertices[0]);
                                eprintln!("  Last vertex: {:?}", vertices[vertices.len()-1]);
                            }
                        },
                        _ => {
                            eprintln!("  Position attribute: Wrong format {:?}", std::mem::discriminant(attribute));
                        }
                    }
                } else {
                    eprintln!("  No position attribute found!");
                }
                
                // Check indices
                if let Some(indices) = step_asset.mesh.indices() {
                    match indices {
                        bevy::render::mesh::Indices::U32(u32_indices) => {
                            eprintln!("  U32 indices: {} indices", u32_indices.len());
                            if !u32_indices.is_empty() {
                                eprintln!("  First 10 indices: {:?}", &u32_indices[..u32_indices.len().min(10)]);
                            }
                        },
                        bevy::render::mesh::Indices::U16(u16_indices) => {
                            eprintln!("  U16 indices: {} indices", u16_indices.len());
                            if !u16_indices.is_empty() {
                                eprintln!("  First 10 indices: {:?}", &u16_indices[..u16_indices.len().min(10)]);
                            }
                        },
                    }
                } else {
                    eprintln!("  No indices found!");
                }
                
                // Try to create a simple manifold from raw data to see what's wrong
                eprintln!("\n=== RAW DATA EXTRACTION ===");
                if let Some(positions) = step_asset.mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
                    if let bevy::render::mesh::VertexAttributeValues::Float32x3(vertices) = positions {
                        if let Some(indices) = step_asset.mesh.indices() {
                            match indices {
                                bevy::render::mesh::Indices::U32(u32_indices) => {
                                    eprintln!("  Attempting to create manifold with {} vertices and {} indices", vertices.len(), u32_indices.len());
                                    
                                    // Flatten vertices
                                    let flattened_vertices: Vec<f32> = vertices.iter().flat_map(|v| [v[0], v[1], v[2]]).collect();
                                    eprintln!("  Flattened to {} float values", flattened_vertices.len());
                                    
                                    // Try creating mesh directly
                                    let mesh = manifold_rs::Mesh::new(&flattened_vertices, u32_indices);
                                    eprintln!("  Created manifold-rs mesh with {} vertices, {} indices", 
                                             mesh.vertices().len() / 3, mesh.indices().len());
                                    
                                    let manifold = manifold_rs::Manifold::from_mesh(mesh);
                                    let mesh_info = manifold.to_mesh();
                                    eprintln!("  Resulting manifold has {} vertices, {} indices", 
                                             mesh_info.vertices().len(), mesh_info.indices().len());
                                },
                                _ => eprintln!("  Unsupported index format"),
                            }
                        }
                    }
                }
            }
        }
        
        debug_state.step_processed = true;
        eprintln!("\n=== DEBUG COMPLETE ===");
        
        // Exit the app by stopping the runner
        std::process::exit(0);
    }
}