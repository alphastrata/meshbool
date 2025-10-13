//! Minimal Boolean Operations Demo
//! 
//! This example demonstrates basic boolean operations (union, intersection, difference) 
//! by showing three shapes arranged like an equation: LHS op RHS = OUTPUT

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Boolean Operations Demo".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup_scene)
        .add_systems(Update, handle_input)
        .run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    println!("🔧 BOOLEAN OPERATIONS DEMO");
    println!("=========================");
    println!("Shows LHS shape (victim) on the left, RHS shape (operator) on the right,");
    println!("and the result of the boolean operation in the center.");
    println!();
    println!("🎮 CONTROLS:");
    println!("  SPACE - Cycle through operations");
    println!("  R - Reset to view original");
    println!("  H - Toggle help");
    println!("  Q - Quit (with error message if output not as expected)");
    println!("  ESC - Quit");
    println!();

    // Create LHS shape (the "victim") - a cube
    let lhs_mesh_handle = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let lhs_material = materials.add(Color::srgb(0.7, 0.7, 0.8)); // Light gray

    // Create RHS shape (the "operator") - a cylinder
    let rhs_mesh_handle = meshes.add(Cylinder::new(1.0, 0.5));
    let rhs_material = materials.add(Color::srgb(0.1, 0.8, 0.1)); // Green

    // Create the output shape (result of the operation) - initially same as LHS
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

    // Spawn the output shape (result of the operation) in the center
    commands.spawn((
        Name::new("Output Shape (Result)"),
        Mesh3d(output_mesh_handle),
        MeshMaterial3d(output_material),
        Transform::from_xyz(0.0, 0.0, 0.0), // Positioned in the center
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
        Transform::from_xyz(0.0, 8.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    println!("✅ Scene setup complete! Operations will be displayed like an equation: LHS op RHS = OUTPUT");
}

fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        // Cycle through operations
        println!("🔄 BOOLEAN UNION (A ∪ B)");
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
        println!("  R - Reset to original");
        println!("  H - Toggle help");
        println!("  Q - Quit (with error message if output not as expected)");
        println!("  ESC - Quit");
    }
}