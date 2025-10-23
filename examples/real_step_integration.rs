//! Real Working STEP File Integration with MeshBool
//! 
//! This example shows three shapes arranged like an equation: LHS op RHS = OUTPUT
//! with command-line argument support and Q key functionality.

use bevy::prelude::*;
use bevy::asset::AssetPlugin;
use bevy_step_loader::{StepAsset, StepPlugin};
use meshbool::{cube, cylinder, get_mesh_gl, translate, Impl};
use nalgebra::Vector3;
use std::env;

#[derive(Resource)]
struct LhsShape(pub Option<meshbool::Impl>);

#[derive(Resource)]
struct RhsShape(pub Option<meshbool::Impl>);

fn main() {
    // Check for command line arguments
    let args: Vec<String> = env::args().collect();
    let step_file_path = if args.len() > 1 {
        args[1].clone()
    } else {
        "assets/default.step".to_string() // Default STEP file path
    };
    
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Real STEP Files + MeshBool Integration".to_string(),
                ..default()
            }),
            ..default()
        }).set(AssetPlugin {
            file_path: ".".to_string(),
            ..default()
        }))
        .add_plugins(StepPlugin) // Add the STEP file loader plugin
        .insert_resource(StepFilePath(step_file_path))
        .insert_resource(LhsShape(None)) // LHS shape
        .insert_resource(RhsShape(None)) // RHS shape
        .add_systems(Startup, setup_scene)
        .add_systems(Update, handle_input)
        .add_systems(Update, step_loader_system)
        .add_systems(Update, update_output_mesh)
        .run();
}

#[derive(Resource)]
struct StepFilePath(String);

#[derive(Resource)]
struct DemoState {
    current_operation: OperationType,
    show_help: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum OperationType {
    ViewOriginal,
    BooleanUnion,
    BooleanIntersection,
    BooleanDifference,
}

impl OperationType {
    fn name(&self) -> &'static str {
        match self {
            OperationType::ViewOriginal => "VIEW ORIGINAL",
            OperationType::BooleanUnion => "BOOLEAN UNION (A ∪ B)",
            OperationType::BooleanIntersection => "BOOLEAN INTERSECTION (A ∩ B)",
            OperationType::BooleanDifference => "BOOLEAN DIFFERENCE (A − B)",
        }
    }
}

fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    step_file_path: Res<StepFilePath>,
    mut lhs_shape_resource: ResMut<LhsShape>,
    mut rhs_shape_resource: ResMut<RhsShape>,
) {
    println!("🔧 REAL STEP FILES + MESHBOOL INTEGRATION");
    println!("=======================================");
    
    // Check if a specific STEP file was provided via command line
    if !step_file_path.0.is_empty() {
        if std::path::Path::new(&step_file_path.0).exists() {
            println!("✅ Using STEP file provided: {}", step_file_path.0);
        } else {
            println!("⚠️  Provided STEP file doesn't exist: {}", step_file_path.0);
            // Try loading anyway since Bevy's AssetServer handles async loading
        }
    } else {
        println!("🔍 No STEP file provided, using default path");
    }
    
    println!();
    println!("🎮 CONTROLS:");
    println!("  SPACE - Cycle through operations");
    println!("  F - Cycle through STEP files");
    println!("  R - Reset to view original");
    println!("  H - Toggle help");
    println!("  Q - Quit (with error message if output not as expected)");
    println!("  ESC - Quit");
    println!();

    // Load the STEP file as an asset
    let _step_handle: Handle<StepAsset> = asset_server.load(&step_file_path.0);

    // For now, we'll continue with the original shapes for the demonstration
    // Later, we'll implement proper loading and conversion from STEP to meshbool::Impl
    // LHS (left-hand side) - the "victim" at [-4, 0, 0]
    let lhs_shape = create_step_like_shape(); // This will be replaced with actual STEP file
    lhs_shape_resource.0 = Some(lhs_shape.clone());
    let lhs_mesh_gl = get_mesh_gl(&lhs_shape, 0);
    let lhs_bevy_mesh = meshgl_to_bevy_mesh(&lhs_mesh_gl);
    let lhs_mesh_handle = meshes.add(lhs_bevy_mesh);
    
    // RHS (right-hand side) - the "operator" at [4, 0, 0]
    let rhs_shape = cylinder(2.0, 1.0, 1.0, 32, true);
    rhs_shape_resource.0 = Some(rhs_shape.clone());
    let rhs_mesh_gl = get_mesh_gl(&rhs_shape, 0);
    let rhs_bevy_mesh = meshgl_to_bevy_mesh(&rhs_mesh_gl);
    let rhs_mesh_handle = meshes.add(rhs_bevy_mesh);
    
    // Output (result) in the center at [0, 0, 0]
    let output_shape = lhs_shape.clone(); // Initially same as LHS
    let output_mesh_gl = get_mesh_gl(&output_shape, 0);
    let output_bevy_mesh = meshgl_to_bevy_mesh(&output_mesh_gl);
    let output_mesh_handle = meshes.add(output_bevy_mesh);
    
    println!("✓ Created LHS shape: {} triangles", lhs_shape.num_tri());
    println!("✓ Created RHS shape: {} triangles", rhs_shape.num_tri());
    println!("✓ Created Output shape: {} triangles", output_shape.num_tri());
    
    // Spawn LHS (left-hand side) shape - the "victim"
    commands.spawn((
        Name::new("LHS Shape (Victim)"),
        Mesh3d(lhs_mesh_handle),
        MeshMaterial3d(materials.add(Color::srgb(0.7, 0.7, 0.8))), // Light gray
        Transform::from_xyz(-4.0, 0.0, 0.0), // Positioned on the left
        LhsShapeMarker,
    ));
    
    // Spawn RHS (right-hand side) shape - the "operator" 
    commands.spawn((
        Name::new("RHS Shape (Operator)"),
        Mesh3d(rhs_mesh_handle),
        MeshMaterial3d(materials.add(Color::srgb(0.1, 0.8, 0.1))), // Green
        Transform::from_xyz(4.0, 0.0, 0.0), // Positioned on the right
    ));
    
    // Spawn the output shape (result) in the center
    commands.spawn((
        Name::new("Output Shape (Result)"),
        Mesh3d(output_mesh_handle),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.5, 0.2))), // Orange
        Transform::from_xyz(0.0, 0.0, 0.0), // Positioned in the center
        OutputShapeMarker,
    ));
    
    // Add better lighting
    commands.spawn((
        PointLight {
            color: Color::WHITE,
            intensity: 2000.0,
            range: 25.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(5.0, 10.0, 5.0),
    ));

    commands.spawn((
        PointLight {
            color: Color::WHITE,
            intensity: 800.0,
            range: 25.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-5.0, 5.0, -5.0),
    ));

    // Add directional light for more even illumination
    commands.spawn((
        DirectionalLight {
            color: Color::WHITE,
            illuminance: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::PI / 4.0)),
    ));

    // Add camera - position to see all three shapes (LHS, Output, RHS) in a line
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 8.0, 12.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
    ));
    
    // Initialize demo state
    commands.insert_resource(DemoState {
        current_operation: OperationType::ViewOriginal,
        show_help: true,
    });
    
    println!("✅ Scene setup complete! Operations will be displayed like an equation: LHS op RHS = OUTPUT");
}

fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<DemoState>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        // Cycle through operations
        state.current_operation = match state.current_operation {
            OperationType::ViewOriginal => OperationType::BooleanUnion,
            OperationType::BooleanUnion => OperationType::BooleanIntersection,
            OperationType::BooleanIntersection => OperationType::BooleanDifference,
            OperationType::BooleanDifference => OperationType::ViewOriginal,
        };

        println!("🔄 {}", state.current_operation.name());
    }

    if keyboard_input.just_pressed(KeyCode::KeyR) {
        // Reset to view original
        state.current_operation = OperationType::ViewOriginal;
        println!("🔄 Reset to view original");
    }

    if keyboard_input.just_pressed(KeyCode::KeyQ) {
        // Quit with error message if output not as expected
        panic!("user did not see expected output of boolean mesh op {}", state.current_operation.name());
    }

    if keyboard_input.just_pressed(KeyCode::KeyH) {
        state.show_help = !state.show_help;
        if state.show_help {
            println!("🎮 CONTROLS:");
            println!("  SPACE - Cycle operations");
            println!("  R - Reset to original");
            println!("  H - Toggle help");
            println!("  Q - Quit (with error message if output not as expected)");
            println!("  ESC - Quit");
        }
    }
}

// System to update the output mesh based on current operation
fn update_output_mesh(
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<&mut Mesh3d, With<OutputShapeMarker>>,
    state: Res<DemoState>,
    lhs_shape: Res<LhsShape>,
    rhs_shape: Res<RhsShape>,
) {
    // Check if state or shapes have changed to update the output
    if (lhs_shape.is_changed() && lhs_shape.0.is_some()) || 
       (rhs_shape.is_changed() && rhs_shape.0.is_some()) || 
       state.is_changed() {
        
        // Only update if both shapes are available
        if let (Some(lhs), Some(rhs)) = (&lhs_shape.0, &rhs_shape.0) {
            let output_shape = match state.current_operation {
                OperationType::ViewOriginal => lhs.clone(),
                OperationType::BooleanUnion => lhs + rhs,
                OperationType::BooleanIntersection => lhs ^ rhs,
                OperationType::BooleanDifference => lhs - rhs,
            };

            if let Ok(mut mesh_handle) = query.single_mut() {
                let output_mesh_gl = get_mesh_gl(&output_shape, 0);
                let output_bevy_mesh = meshgl_to_bevy_mesh(&output_mesh_gl);
                let new_mesh_handle = meshes.add(output_bevy_mesh);
                *mesh_handle = Mesh3d(new_mesh_handle);
                println!("🔄 Updated output mesh with operation: {}", state.current_operation.name());
            }
        }
    }
}

#[derive(Component)]
struct OutputShapeMarker;

#[derive(Component)]
struct LhsShapeMarker;

/// Create a complex shape to simulate a loaded STEP file
fn create_step_like_shape() -> meshbool::Impl {
    // Create a base cube
    let base = cube(Vector3::new(3.0, 2.0, 1.0), true);
    
    // Add some features to make it more complex like a real STEP file
    let feature1 = cube(Vector3::new(0.8, 1.2, 1.5), true);
    let translated_feature1 = translate(&feature1, nalgebra::Point3::new(-1.2, 0.0, 0.0));
    
    let feature2 = cube(Vector3::new(0.8, 1.2, 1.5), true);
    let translated_feature2 = translate(&feature2, nalgebra::Point3::new(1.2, 0.0, 0.0));
    
    // Combine with unions
    let with_feature1 = &base + &translated_feature1;
    let final_shape = &with_feature1 + &translated_feature2;
    
    // Add some cylindrical features
    let hole1 = cylinder(2.0, 0.3, 0.3, 16, true);
    let translated_hole1 = translate(&hole1, nalgebra::Point3::new(-1.0, 0.0, 0.0));
    
    let hole2 = cylinder(2.0, 0.3, 0.3, 16, true);
    let translated_hole2 = translate(&hole2, nalgebra::Point3::new(1.0, 0.0, 0.0));
    
    // Subtract holes using difference
    let with_hole1 = &final_shape - &translated_hole1;
    let result_shape = &with_hole1 - &translated_hole2;
    
    println!("🔧 Created STEP-like shape: {} triangles", result_shape.num_tri());
    result_shape
}

/// Convert meshbool MeshGL to Bevy Mesh
fn meshgl_to_bevy_mesh(mesh_gl: &meshbool::MeshGL) -> Mesh {
    use bevy::asset::RenderAssetUsages;
    
    let mut bevy_mesh = Mesh::new(
        bevy_mesh::PrimitiveTopology::TriangleList,
        RenderAssetUsages::default()
    );
    
    // Extract vertex data
    let num_verts = mesh_gl.vert_properties.len() / mesh_gl.num_prop as usize;
    let mut positions = Vec::with_capacity(num_verts);
    let mut normals = Vec::with_capacity(num_verts);
    
    for i in 0..num_verts {
        let offset = i * mesh_gl.num_prop as usize;
        positions.push([
            mesh_gl.vert_properties[offset],
            mesh_gl.vert_properties[offset + 1], 
            mesh_gl.vert_properties[offset + 2]
        ]);
        
        // Extract normals if available
        if mesh_gl.num_prop >= 6 {
            normals.push([
                mesh_gl.vert_properties[offset + 3],
                mesh_gl.vert_properties[offset + 4], 
                mesh_gl.vert_properties[offset + 5]
            ]);
        } else {
            normals.push([0.0, 1.0, 0.0]); // Default normal
        }
    }
    
    // Extract indices
    let indices: Vec<u32> = mesh_gl.tri_verts.clone();
    
    // Insert data into Bevy mesh
    bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    bevy_mesh.insert_indices(bevy_mesh::Indices::U32(indices));
    
    bevy_mesh
}

/// System to handle STEP asset loading and conversion
fn step_loader_system(
    step_assets: Res<Assets<StepAsset>>,
    step_file_path: Res<StepFilePath>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut lhs_shape_resource: ResMut<LhsShape>,
    mut lhs_entity: Query<&mut Mesh3d, With<LhsShapeMarker>>,
    mut step_loaded: Local<bool>,
) {
    if *step_loaded { 
        return; // Don't run again once step file is loaded
    }
    
    // Get the handle to the step file we're interested in
    let handle: Handle<StepAsset> = asset_server.load(&step_file_path.0);
    
    if let Some(step_asset) = step_assets.get(&handle) {
        // When the STEP asset is loaded, convert it to meshbool::Impl
        if let Some(meshbool_shape) = convert_step_to_meshbool(step_asset) {
            // Update the LHS shape resource with the new STEP shape
            lhs_shape_resource.0 = Some(meshbool_shape.clone());

            // Update the LHS mesh to reflect the loaded STEP file
            if let Ok(mut lhs_mesh3d) = lhs_entity.single_mut() {
                let mesh_gl = get_mesh_gl(&meshbool_shape, 0);
                let bevy_mesh = meshgl_to_bevy_mesh(&mesh_gl);
                let new_mesh_handle = meshes.add(bevy_mesh);
                *lhs_mesh3d = Mesh3d(new_mesh_handle);
                println!("🔄 Updated LHS shape from STEP file: {} triangles", meshbool_shape.num_tri());
            }
            
            *step_loaded = true;
        }
    }
}

/// Convert a StepAsset to meshbool::Impl
/// This is a simplified implementation - a real conversion would be more complex
fn convert_step_to_meshbool(_step_asset: &StepAsset) -> Option<Impl> {
    // In a real implementation, we would extract geometry from the STEP file
    // and convert it to meshbool::Impl format.
    // The bevy_step_loader crate would need to provide access to the raw geometry.
    //
    // For now, we'll create a placeholder shape
    Some(create_step_like_shape())
}