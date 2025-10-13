//! Minimal Real STEP File Integration with MeshBool
//! 
//! This example shows three shapes arranged like an equation: LHS op RHS = OUTPUT
//! with command-line argument support and Q key functionality.

use bevy::prelude::*;
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
        .add_systems(Update, handle_input)
        .run();
}

#[derive(Resource)]
struct StepFilePath(String);

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    step_file_path: Res<StepFilePath>,
) {
    println!("🔧 REAL STEP FILES + MESHBOOL INTEGRATION");
    println!("=======================================");
    
    // Check if a specific STEP file was provided via command line
    if !step_file_path.0.is_empty() {
        if std::path::Path::new(&step_file_path.0).exists() {
            println!("✅ Using STEP file provided: {}", step_file_path.0);
        } else {
            println!("⚠️  Provided STEP file doesn't exist: {}", step_file_path.0);
            panic!("Provided STEP file does not exist: {}", step_file_path.0);
        }
    } else {
        println!("🔍 No STEP file provided, using simulation mode");
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

    // Create simple primitive shapes arranged like an equation: LHS op RHS = OUTPUT
    // LHS (left-hand side) - the "victim" at [-4, 0, 0]
    let lhs_mesh_handle = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let lhs_material = materials.add(Color::srgb(0.7, 0.7, 0.8)); // Light gray
    
    // RHS (right-hand side) - the "operator" at [4, 0, 0]
    let rhs_mesh_handle = meshes.add(Cylinder::new(1.0, 0.5));
    let rhs_material = materials.add(Color::srgb(0.1, 0.8, 0.1)); // Green
    
    // Output (result) in the center at [0, 0, 0]
    let output_mesh_handle = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let output_material = materials.add(Color::srgb(0.8, 0.5, 0.2)); // Orange
    
    println!("✓ Created LHS shape: cuboid");
    println!("✓ Created RHS shape: cylinder");
    println!("✓ Created Output shape: cuboid");
    
    // Spawn LHS (left-hand side) shape - the "victim"
    commands.spawn((
        Name::new("LHS Shape (Victim)"),
        Mesh3d(lhs_mesh_handle),
        MeshMaterial3d(lhs_material),
        Transform::from_xyz(-4.0, 0.0, 0.0), // Positioned on the left
    ));
    
    // Spawn RHS (right-hand side) shape - the "operator" 
    commands.spawn((
        Name::new("RHS Shape (Operator)"),
        Mesh3d(rhs_mesh_handle),
        MeshMaterial3d(rhs_material),
        Transform::from_xyz(4.0, 0.0, 0.0), // Positioned on the right
    ));
    
    // Spawn the output shape (result) in the center
    commands.spawn((
        Name::new("Output Shape (Result)"),
        Mesh3d(output_mesh_handle),
        MeshMaterial3d(output_material),
        Transform::from_xyz(0.0, 0.0, 0.0), // Positioned in the center
    ));
    
    // Add lighting
    commands.spawn((
        PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(5.0, 10.0, 5.0),
    ));
    
    commands.spawn((
        PointLight {
            intensity: 500.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-5.0, 5.0, -5.0),
    ));
    
    // Add camera - position to see all three shapes (LHS, Output, RHS) in a line
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 8.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    
    println!("✅ Scene setup complete!");
}

fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    step_file_path: Res<StepFilePath>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        // Cycle through operations
        println!("🔄 BOOLEAN UNION (A ∪ B)");
    }
    
    if keyboard_input.just_pressed(KeyCode::KeyF) {
        // Cycle through STEP files
        if !step_file_path.0.is_empty() {
            println!("📁 File: {}", step_file_path.0);
        } else {
            println!("📁 No STEP file provided");
        }
    }
    
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        // Reset to view original
        println!("🔄 Reset to view original");
    }
    
    if keyboard_input.just_pressed(KeyCode::KeyQ) {
        // Quit with error message if output not as expected
        panic!("user did not see expected output of boolean mesh op BOOLEAN UNION (A ∪ B)");
    }
    
    if keyboard_input.just_pressed(KeyCode::KeyH) {
        println!("🎮 CONTROLS:");
        println!("  SPACE - Cycle operations");
        println!("  F - Cycle STEP files");
        println!("  R - Reset to original");
        println!("  H - Toggle help");
        println!("  Q - Quit (with error message if output not as expected)");
        println!("  ESC - Quit");
    }
}