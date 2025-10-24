use bevy::prelude::*;
use bevy_step_loader::{StepAsset, StepPlugin}; // Import our plugin and asset type

fn main() {
    // Get the STEP file path from command line args
    let step_file = std::env::args().nth(1).unwrap_or_else(|| "real_parts/multifeature.step".to_string());
    
    App::new()
        .add_plugins((DefaultPlugins.set(ImagePlugin::default_nearest()), StepPlugin)) // Add our plugin
        .insert_resource(StepFileResource(step_file))
        .add_systems(Startup, setup)
        .add_systems(Update, log_step_mesh_stats)
        .run();
}

#[derive(Resource)]
struct StepFileResource(String);

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    step_file_resource: Res<StepFileResource>,
) {
    let step_file = &step_file_resource.0;
    
    // Load the STEP file specified via command line
    let step_handle: Handle<StepAsset> = asset_server.load(step_file);
    
    // Store the handle to check for loading in the update system
    commands.insert_resource(StepHandleResource(step_handle));
    
    // Camera and light setup
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

#[derive(Resource)]
struct StepHandleResource(Handle<StepAsset>);

fn log_step_mesh_stats(
    step_assets: Res<Assets<StepAsset>>,
    step_handle_resource: Res<StepHandleResource>,
) {
    if let Some(step_asset) = step_assets.get(&step_handle_resource.0) {
        // Get the mesh data
        let mesh = &step_asset.mesh;
        
        // Get positions (vertices)
        let positions = if let Some(positions) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            match positions {
                bevy::render::mesh::VertexAttributeValues::Float32x3(pos) => {
                    eprintln!("STEP Mesh Statistics:");
                    eprintln!("  Vertices: {}", pos.len());
                    pos
                },
                _ => {
                    eprintln!("STEP Mesh: Position attribute is not Float32x3");
                    return;
                }
            }
        } else {
            eprintln!("STEP Mesh: No position attribute found");
            return;
        };
        
        // Get indices (triangles)
        let indices = if let Some(indices) = mesh.indices() {
            match indices {
                bevy::render::mesh::Indices::U32(indices_vec) => {
                    eprintln!("  Indices: {}", indices_vec.len());
                    eprintln!("  Triangles: {}", indices_vec.len() / 3);
                    indices_vec.len()
                },
                bevy::render::mesh::Indices::U16(indices_vec) => {
                    eprintln!("  Indices: {}", indices_vec.len());
                    eprintln!("  Triangles: {}", indices_vec.len() / 3);
                    indices_vec.len()
                },
                _ => {
                    eprintln!("STEP Mesh: Indices are not U32 or U16");
                    0
                }
            }
        } else {
            eprintln!("  Indices: 0 (mesh is unindexed)");
            eprintln!("  Triangles: {}", positions.len() / 3); // assuming each 3 vertices form a triangle
            0
        };
        
        // Calculate approximate edges
        // This is a rough calculation - in a triangulated mesh, each triangle has 3 edges
        // but edges are shared between adjacent triangles
        let rough_edge_estimate = if indices > 0 {
            indices  // This is a very rough estimate, actual edges would require topology analysis
        } else {
            positions.len() // For unindexed mesh
        };
        
        eprintln!("  Rough edge estimate: ~{}", rough_edge_estimate);
        eprintln!("  Topology: {:?}", mesh.primitive_topology());
        
        // We can't easily calculate exact edge count without analyzing mesh topology,
        // but we have vertices and triangle counts which are the most important
        
        // Remove the resource to prevent further logging
        // This is a simple approach - in a real app you might want to have a flag system
        // For now, just exit after first calculation
        std::process::exit(0);
    }
}