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
        println!("=== STEP FILE LOADED ===");
        println!("Asset ID: {:?}", handle.0.id());
        
        // Get mesh statistics
        let vertex_count = if let Some(positions) = asset.mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            match positions {
                bevy::render::mesh::VertexAttributeValues::Float32x3(pos) => pos.len(),
                _ => 0,
            }
        } else {
            0
        };
        
        let index_count = if let Some(indices) = asset.mesh.indices() {
            match indices {
                bevy::render::mesh::Indices::U32(indices_vec) => indices_vec.len(),
                bevy::render::mesh::Indices::U16(indices_vec) => indices_vec.len(),
            }
        } else {
            0
        };
        
        println!("Vertices: {}", vertex_count);
        println!("Indices: {}", index_count);
        println!("Triangles: {}", index_count / 3);
        
        // Debug vertex data
        if let Some(positions) = asset.mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            match positions {
                bevy::render::mesh::VertexAttributeValues::Float32x3(vertices) => {
                    println!("First 5 vertices:");
                    for (i, vertex) in vertices.iter().take(5).enumerate() {
                        println!("  {}: [{:.3}, {:.3}, {:.3}]", i, vertex[0], vertex[1], vertex[2]);
                    }
                    println!("Last 5 vertices:");
                    for (i, vertex) in vertices.iter().skip(vertices.len().saturating_sub(5)).enumerate() {
                        println!("  {}: [{:.3}, {:.3}, {:.3}]", i, vertex[0], vertex[1], vertex[2]);
                    }
                }
                _ => println!("Unknown vertex format"),
            }
        }
        
        // Debug indices
        if let Some(indices) = asset.mesh.indices() {
            match indices {
                bevy::render::mesh::Indices::U32(indices_vec) => {
                    println!("First 15 indices:");
                    for (i, index) in indices_vec.iter().take(15).enumerate() {
                        print!("{} ", index);
                        if (i + 1) % 15 == 0 { println!(); }
                    }
                }
                bevy::render::mesh::Indices::U16(indices_vec) => {
                    println!("First 15 indices:");
                    for (i, index) in indices_vec.iter().take(15).enumerate() {
                        print!("{} ", index);
                        if (i + 1) % 15 == 0 { println!(); }
                    }
                }
            }
        }
        
        std::process::exit(0);
    }
}