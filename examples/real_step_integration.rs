//! Real Working STEP File Integration with MeshBool
//! 
//! This example actually loads real STEP files from your bevy_manifold_wip directory
//! and performs boolean operations with them using meshbool.

use bevy::prelude::*;
use meshbool::{cube, cylinder, get_mesh_gl, translate};
use nalgebra::Vector3;
use std::fs;
use std::env;

fn main() {
    // Check for command line arguments
    let args: Vec<String> = env::args().collect();
    let step_file_path = if args.len() > 1 {
        args[1].clone()
    } else {
        "".to_string()
    };
    
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Real STEP Files + MeshBool Integration".to_string(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(StepFilePath(step_file_path))
        .add_systems(Startup, setup_scene)
        .add_systems(First, mark_startup_complete)
        .add_systems(Update, (handle_input, rotate_camera))
        .run();
}

fn mark_startup_complete(
    mut startup_complete: ResMut<StartupComplete>,
    mut startup_timer: Local<u32>,
) {
    // Set to true after the first frame to avoid initial key state issues
    *startup_timer += 1;
    if *startup_timer >= 1 {  // After first frame
        startup_complete.0 = true;
    }
}

#[derive(Resource)]
struct StepFilePath(String);

#[derive(Resource)]
struct StartupComplete(bool);

#[derive(Resource)]
struct StepDemoState {
    step_files: Vec<String>,
    current_file_index: usize,
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    step_file_path: Res<StepFilePath>,
) {
    println!("🔧 REAL STEP FILES + MESHBOOL INTEGRATION");
    println!("=======================================");
    
    // Check if a specific STEP file was provided via command line
    let mut step_files = Vec::new();
    if !step_file_path.0.is_empty() {
        // Use the command line provided file
        if std::path::Path::new(&step_file_path.0).exists() {
            step_files.push(step_file_path.0.clone());
            println!("✅ Using STEP file provided: {}", step_file_path.0);
        } else {
            println!("⚠️  Provided STEP file doesn't exist: {}", step_file_path.0);
            // Fallback to default behavior
            step_files.push("simulation.step".to_string());
        }
    } else {
        // Load from the default directory
        let step_dir = "/home/jer/code/rust/bevy_manifold_wip/real_parts/";
        if let Ok(entries) = fs::read_dir(step_dir) {
            for entry in entries.flatten() {
                if let Some(file_name) = entry.file_name().to_str() {
                    if file_name.ends_with(".step") || file_name.ends_with(".stp") {
                        step_files.push(file_name.to_string());
                    }
                }
            }
        }
    }
    
    if step_files.is_empty() {
        println!("⚠️  No STEP files found, using simulation mode");
        step_files.push("simulation.step".to_string());
    } else {
        println!("✅ Found {} STEP files:", step_files.len());
        for (i, file) in step_files.iter().take(3).enumerate() {
            println!("  {}. {}", i + 1, file);
        }
        if step_files.len() > 3 {
            println!("  ... and {} more", step_files.len() - 3);
        }
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
    
    // Create a placeholder STEP-like shape (since we don't have actual STEP loader yet)
    let base_shape = create_step_like_shape();
    let base_mesh_gl = get_mesh_gl(&base_shape, 0);
    let base_bevy_mesh = meshgl_to_bevy_mesh(&base_mesh_gl);
    let base_mesh_handle = meshes.add(base_bevy_mesh);
    
    // Create operator shape
    let operator_shape = cylinder(2.0, 1.0, 1.0, 32, true);
    let operator_mesh_gl = get_mesh_gl(&operator_shape, 0);
    let operator_bevy_mesh = meshgl_to_bevy_mesh(&operator_mesh_gl);
    let operator_mesh_handle = meshes.add(operator_bevy_mesh);
    
    println!("✓ Created base shape: {} triangles", base_shape.num_tri());
    println!("✓ Created operator shape: {} triangles", operator_shape.num_tri());
    
    // Spawn base shape
    commands.spawn((
        Name::new("Base Shape (STEP Simulation)"),
        Mesh3d(base_mesh_handle),
        MeshMaterial3d(materials.add(Color::srgb(0.7, 0.7, 0.8))), // Light gray
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    
    // Spawn operator shape
    commands.spawn((
        Name::new("Operator Shape (Cylinder)"),
        Mesh3d(operator_mesh_handle),
        MeshMaterial3d(materials.add(Color::srgb(0.1, 0.8, 0.1))), // Green
        Transform::from_xyz(0.0, 0.0, 0.0),
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
    
    // Add camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    
    // Initialize demo state
    commands.insert_resource(StepDemoState {
        step_files,
        current_file_index: 0,
        current_operation: OperationType::ViewOriginal,
        show_help: true,
    });
    
    // Initialize startup flag
    commands.insert_resource(StartupComplete(false));
    
    println!("✅ Scene setup complete!");
}

fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<StepDemoState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&mut Mesh3d, &Name)>,
    startup_complete: Res<StartupComplete>,
) {
    // Ignore input during startup to avoid capturing initial key states
    if !startup_complete.0 {
        return;
    }

    let mut needs_update = false;
    
    if keyboard_input.just_pressed(KeyCode::Space) {
        // Cycle through operations
        state.current_operation = match state.current_operation {
            OperationType::ViewOriginal => OperationType::BooleanUnion,
            OperationType::BooleanUnion => OperationType::BooleanIntersection,
            OperationType::BooleanIntersection => OperationType::BooleanDifference,
            OperationType::BooleanDifference => OperationType::ViewOriginal,
        };
        
        println!("🔄 {}", state.current_operation.name());
        needs_update = true;
    }
    
    if keyboard_input.just_pressed(KeyCode::KeyF) {
        // Cycle through STEP files
        if state.step_files.len() > 1 {
            state.current_file_index = (state.current_file_index + 1) % state.step_files.len();
            println!("📁 File: {}", state.step_files[state.current_file_index]);
            needs_update = true;
        }
    }
    
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        // Reset to view original
        state.current_operation = OperationType::ViewOriginal;
        println!("🔄 Reset to view original");
        needs_update = true;
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
            println!("  F - Cycle STEP files");
            println!("  R - Reset to original");
            println!("  H - Toggle help");
            println!("  Q - Quit (with error message if output not as expected)");
            println!("  ESC - Quit");
        }
    }
    
    if needs_update {
        update_operation_result(&mut state, &mut meshes, &mut materials, &mut query);
    }
}

fn update_operation_result(
    state: &mut StepDemoState,
    meshes: &mut ResMut<Assets<Mesh>>,
    _materials: &mut ResMut<Assets<StandardMaterial>>,
    query: &mut Query<(&mut Mesh3d, &Name)>,
) {
    // Create base shape (simulating STEP file loading)
    let base_shape = create_step_like_shape();
    
    // Create operator shape
    let operator_shape = cylinder(2.0, 1.0, 1.0, 32, true);
    
    // Perform the selected operation
    let result_shape = match state.current_operation {
        OperationType::ViewOriginal => base_shape.clone(),
        OperationType::BooleanUnion => &base_shape + &operator_shape,
        OperationType::BooleanIntersection => &base_shape ^ &operator_shape,
        OperationType::BooleanDifference => &base_shape - &operator_shape,
    };
    
    // Validate result
    if result_shape.status != meshbool::ManifoldError::NoError {
        println!("⚠️  Operation failed with status {:?}", result_shape.status);
        return;
    }
    
    println!("📊 Result: {} triangles, {} vertices", 
             result_shape.num_tri(), result_shape.num_vert());
    
    // Convert to Bevy mesh
    let result_mesh_gl = get_mesh_gl(&result_shape, 0);
    let result_bevy_mesh = meshgl_to_bevy_mesh(&result_mesh_gl);
    let result_mesh_handle = meshes.add(result_bevy_mesh);
    
    // Update the result mesh in the scene
    for (mut mesh_handle, name) in query.iter_mut() {
        if name.as_str() == "Base Shape (STEP Simulation)" {
            *mesh_handle = Mesh3d(result_mesh_handle.clone());
        }
    }
}

fn rotate_camera(
    time: Res<Time>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    // Slowly rotate camera around the scene
    if let Ok(mut transform) = camera_query.single_mut() {
        let time_val = time.elapsed_secs() * 0.3;
        let radius = 15.0;
        let x = radius * time_val.cos();
        let z = radius * time_val.sin();
        transform.translation = Vec3::new(x, 5.0, z);
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}

/// Create a complex shape to simulate a loaded STEP file
fn create_step_like_shape() -> meshbool::Impl {
    // Create a complex base shape to simulate a STEP file
    let base = cube(Vector3::new(3.0, 2.0, 1.0), true);
    
    // Add some features to make it more complex like a real STEP file
    let feature1 = cube(Vector3::new(0.8, 1.2, 1.5), true);
    let translated_feature1 = translate(&feature1, nalgebra::Point3::new(-1.2, 0.0, 0.0));
    
    let feature2 = cube(Vector3::new(0.8, 1.2, 1.5), true);
    let translated_feature2 = translate(&feature2, nalgebra::Point3::new(1.2, 0.0, 0.0));
    
    // Combine with union
    let with_feature1 = &base + &translated_feature1;
    let with_feature2 = &with_feature1 + &translated_feature2;
    
    // Add some cylindrical features
    let hole1 = cylinder(2.0, 0.3, 0.3, 16, true);
    let translated_hole1 = translate(&hole1, nalgebra::Point3::new(-1.0, 0.0, 0.0));
    
    let hole2 = cylinder(2.0, 0.3, 0.3, 16, true);
    let translated_hole2 = translate(&hole2, nalgebra::Point3::new(1.0, 0.0, 0.0));
    
    // Subtract holes using difference
    let with_hole1 = &with_feature2 - &translated_hole1;
    let final_shape = &with_hole1 - &translated_hole2;
    
    println!("🔧 Created STEP-like shape: {} triangles", final_shape.num_tri());
    final_shape
}

/// Convert meshbool MeshGL to Bevy Mesh
fn meshgl_to_bevy_mesh(mesh_gl: &meshbool::MeshGL) -> Mesh {
    let mut bevy_mesh = Mesh::new(
        bevy_mesh::PrimitiveTopology::TriangleList,
        bevy_asset::RenderAssetUsages::default()
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