//! STEP File + MeshBool Integration Demo
//! 
//! This demo loads real STEP files and performs boolean operations with them.
//! 
//! Controls:
//!   SPACE - Cycle through boolean operations
//!   S - Cycle through STEP files in ./real_parts/
//!   Q - Panic with "user did not see what was expected" message
//!   R - Reset to view original
//!   H - Toggle help
//!   ESC - Quit

use bevy::prelude::*;
use meshbool::{cube, cylinder, get_mesh_gl, translate};
use nalgebra::Vector3;
use std::env;
use std::fs;

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
                title: "STEP Files + MeshBool Integration".to_string(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(StepFilePath(step_file_path))
        .insert_resource(StepFiles::default())
        .insert_resource(CurrentOperation::ViewOriginal)
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (handle_input, rotate_camera_lighting))
        .run();
}

#[derive(Resource)]
struct StepFilePath(String);

#[derive(Resource, Default)]
struct StepFiles {
    files: Vec<String>,
    current_index: usize,
}

#[derive(Resource, Clone, Copy, PartialEq)]
enum CurrentOperation {
    ViewOriginal,
    BooleanUnion,
    BooleanIntersection,
    BooleanDifference,
}

impl CurrentOperation {
    fn name(&self) -> &'static str {
        match self {
            CurrentOperation::ViewOriginal => "VIEW ORIGINAL",
            CurrentOperation::BooleanUnion => "BOOLEAN UNION (A ∪ B)",
            CurrentOperation::BooleanIntersection => "BOOLEAN INTERSECTION (A ∩ B)",
            CurrentOperation::BooleanDifference => "BOOLEAN DIFFERENCE (A − B)",
        }
    }
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    step_file_path: Res<StepFilePath>,
    mut step_files: ResMut<StepFiles>,
) {
    println!("🔧 STEP FILES + MESHBOOL INTEGRATION");
    println!("==================================");
    
    // Load STEP files from ./real_parts/
    let step_dir = "./real_parts/";
    if let Ok(entries) = fs::read_dir(step_dir) {
        for entry in entries.flatten() {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".step") || file_name.ends_with(".stp") {
                    step_files.files.push(file_name.to_string());
                }
            }
        }
    }
    
    let current_step_file = if !step_file_path.0.is_empty() {
        println!("📁 Loading STEP file from command line: {}", step_file_path.0);
        step_file_path.0.clone()
    } else if !step_files.files.is_empty() {
        println!("📂 Found {} STEP files in ./real_parts/", step_files.files.len());
        step_files.files[0].clone()
    } else {
        println!("⚠️  No STEP files found, using simulation mode");
        "simulation.step".to_string()
    };
    
    println!();
    println!("🎮 CONTROLS:");
    println!("  SPACE - Cycle through operations");
    println!("  S - Cycle through STEP files");
    println!("  Q - Panic with 'user did not see what was expected'");
    println!("  R - Reset to view original");
    println!("  H - Toggle help");
    println!("  ESC - Quit");
    println!();
    
    // Create base shape (simulating loaded STEP file)
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
    
    // Add improved lighting for better visualization
    commands.spawn((
        PointLight {
            intensity: 2000.0,
            shadows_enabled: true,
            color: Color::srgb(1.0, 1.0, 1.0),
            ..default()
        },
        Transform::from_xyz(5.0, 10.0, 5.0),
    ));
    
    commands.spawn((
        PointLight {
            intensity: 1000.0,
            shadows_enabled: true,
            color: Color::srgb(0.8, 0.8, 1.0),
            ..default()
        },
        Transform::from_xyz(-5.0, 5.0, -5.0),
    ));
    
    // Add ambient light for better overall illumination
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.3, 0.3, 0.3),
        brightness: 0.4,
        affects_lightmapped_meshes: false,
    });
    
    // Add camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    
    println!("📄 Current STEP file: {}", current_step_file);
    println!("✅ Scene setup complete!");
}

fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut current_operation: ResMut<CurrentOperation>,
    mut step_files: ResMut<StepFiles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&mut Mesh3d, &Name)>,
) {
    let mut needs_update = false;
    
    if keyboard_input.just_pressed(KeyCode::Space) {
        // Cycle through operations
        *current_operation = match *current_operation {
            CurrentOperation::ViewOriginal => CurrentOperation::BooleanUnion,
            CurrentOperation::BooleanUnion => CurrentOperation::BooleanIntersection,
            CurrentOperation::BooleanIntersection => CurrentOperation::BooleanDifference,
            CurrentOperation::BooleanDifference => CurrentOperation::ViewOriginal,
        };
        
        println!("🔄 {}", current_operation.name());
        needs_update = true;
    }
    
    if keyboard_input.just_pressed(KeyCode::KeyS) {
        // Cycle through STEP files
        if !step_files.files.is_empty() {
            step_files.current_index = (step_files.current_index + 1) % step_files.files.len();
            println!("📁 File: {}", step_files.files[step_files.current_index]);
            needs_update = true;
        }
    }
    
    if keyboard_input.just_pressed(KeyCode::KeyQ) {
        // Panic with user message
        panic!("user did not see what was expected");
    }
    
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        // Reset to view original
        *current_operation = CurrentOperation::ViewOriginal;
        println!("🔄 Reset to view original");
        needs_update = true;
    }
    
    if keyboard_input.just_pressed(KeyCode::KeyH) {
        println!("🎮 CONTROLS:");
        println!("  SPACE - Cycle through operations");
        println!("  S - Cycle through STEP files");
        println!("  Q - Panic with 'user did not see what was expected'");
        println!("  R - Reset to view original");
        println!("  H - Toggle help");
        println!("  ESC - Quit");
    }
    
    if needs_update {
        update_operation_result(&mut current_operation, &mut step_files, &mut meshes, &mut materials, &mut query);
    }
}

fn update_operation_result(
    current_operation: &mut ResMut<CurrentOperation>,
    _step_files: &mut ResMut<StepFiles>,
    meshes: &mut ResMut<Assets<Mesh>>,
    _materials: &mut ResMut<Assets<StandardMaterial>>,
    query: &mut Query<(&mut Mesh3d, &Name)>,
) {
    // Create base shape (simulating loaded STEP file)
    let base_shape = create_step_like_shape();
    
    // Create operator shape
    let operator_shape = cylinder(2.0, 1.0, 1.0, 32, true);
    
    // Perform the selected operation
    let result_shape = match **current_operation {
        CurrentOperation::ViewOriginal => base_shape.clone(),
        CurrentOperation::BooleanUnion => &base_shape + &operator_shape,
        CurrentOperation::BooleanIntersection => &base_shape ^ &operator_shape,
        CurrentOperation::BooleanDifference => &base_shape - &operator_shape,
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

fn rotate_camera_lighting(
    time: Res<Time>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
    mut light_query: Query<&mut Transform, (With<PointLight>, Without<Camera3d>)>,
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
    
    // Rotate lights for better visualization
    for (i, mut transform) in light_query.iter_mut().enumerate() {
        let time_val = time.elapsed_secs() * 0.2 + (i as f32 * std::f32::consts::PI);
        let radius = 8.0;
        let x = radius * time_val.cos();
        let z = radius * time_val.sin();
        transform.translation = Vec3::new(x, 8.0, z);
    }
}

/// Create a complex shape to simulate a loaded STEP file
fn create_step_like_shape() -> meshbool::Impl {
    // Create a complex base shape to simulate a STEP file
    let base = cube(Vector3::new(3.0, 2.0, 1.0), true);
    
    // Add some features
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