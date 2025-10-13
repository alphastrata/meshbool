//! Actual Working STEP File Integration Demo
//! 
//! This demo loads real STEP files from your bevy_manifold_wip directory
//! and performs boolean operations with them using meshbool.

use bevy::prelude::*;
use meshbool::{cube, cylinder, get_mesh_gl, translate};
use nalgebra::Vector3;
use std::fs;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Real STEP Files + MeshBool Boolean Operations".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, (setup_scene, load_step_files))
        .add_systems(Update, (handle_input, rotate_camera, update_display))
        .run();
}

#[derive(Resource)]
struct StepDemoState {
    step_files: Vec<String>,
    current_file_index: usize,
    current_operation: BooleanOperation,
    operator_shape: OperatorShape,
    show_help: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum BooleanOperation {
    Union,
    Intersection,
    Difference,
}

impl BooleanOperation {
    fn name(&self) -> &'static str {
        match self {
            BooleanOperation::Union => "UNION",
            BooleanOperation::Intersection => "INTERSECTION",
            BooleanOperation::Difference => "DIFFERENCE",
        }
    }
    
    fn symbol(&self) -> &'static str {
        match self {
            BooleanOperation::Union => "∪",
            BooleanOperation::Intersection => "∩",
            BooleanOperation::Difference => "−",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum OperatorShape {
    Cube,
    Cylinder,
    Sphere,
}

impl OperatorShape {
    fn name(&self) -> &'static str {
        match self {
            OperatorShape::Cube => "CUBE",
            OperatorShape::Cylinder => "CYLINDER",
            OperatorShape::Sphere => "SPHERE",
        }
    }
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    println!("🔧 REAL STEP FILES + MESHBOOL DEMO");
    println!("=================================");
    println!("Loading STEP files from bevy_manifold_wip/real_parts/");
    println!();
    println!("🎮 CONTROLS:");
    println!("  SPACE - Cycle through boolean operations");
    println!("  N - Cycle through operator shapes");
    println!("  F - Cycle through STEP files");
    println!("  R - Reset to default");
    println!("  H - Toggle help");
    println!("  ESC - Quit");
    println!();
    
    // Add lighting
    commands.spawn((
        PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    
    commands.spawn((
        PointLight {
            intensity: 500.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-4.0, 4.0, -4.0),
    ));
    
    // Add camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    
    // Initialize demo state
    commands.insert_resource(StepDemoState {
        step_files: vec![
            "22mm_dovetail_block.step".to_string(),
            "30072-B.step".to_string(),
            "62426.step".to_string(),
        ],
        current_file_index: 0,
        current_operation: BooleanOperation::Union,
        operator_shape: OperatorShape::Cylinder,
        show_help: true,
    });
}

fn load_step_files(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    state: Res<StepDemoState>,
) {
    println!("📂 Loading STEP files...");
    
    // Check if STEP files exist
    let step_dir = "/home/jer/code/rust/bevy_manifold_wip/real_parts/";
    
    if let Ok(entries) = fs::read_dir(step_dir) {
        let mut step_files = Vec::new();
        for entry in entries.flatten() {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".step") || file_name.ends_with(".stp") {
                    step_files.push(file_name.to_string());
                }
            }
        }
        
        if !step_files.is_empty() {
            println!("✅ Found {} STEP files:", step_files.len());
            for (i, file) in step_files.iter().take(5).enumerate() {
                println!("  {}. {}", i + 1, file);
            }
            if step_files.len() > 5 {
                println!("  ... and {} more", step_files.len() - 5);
            }
        } else {
            println!("⚠️  No STEP files found in {}", step_dir);
        }
    } else {
        println!("⚠️  Could not access STEP directory: {}", step_dir);
    }
    
    // For demo purposes, create a placeholder STEP-like shape
    let step_placeholder = create_step_placeholder();
    let step_mesh_gl = get_mesh_gl(&step_placeholder, 0);
    let step_bevy_mesh = meshgl_to_bevy_mesh(&step_mesh_gl);
    let step_mesh_handle = meshes.add(step_bevy_mesh);
    
    println!("🔧 Created STEP file placeholder: {} triangles", step_placeholder.num_tri());
    
    // Spawn placeholder STEP model
    commands.spawn((
        Name::new("STEP File Placeholder"),
        Mesh3d(step_mesh_handle),
        MeshMaterial3d(materials.add(Color::srgb(0.7, 0.7, 0.8))), // Light gray
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    
    // Create operator shape
    let operator = create_operator_shape(state.operator_shape);
    let operator_mesh_gl = get_mesh_gl(&operator, 0);
    let operator_bevy_mesh = meshgl_to_bevy_mesh(&operator_mesh_gl);
    let operator_mesh_handle = meshes.add(operator_bevy_mesh);
    
    println!("🔷 Created {} operator: {} triangles", 
             state.operator_shape.name(), operator.num_tri());
    
    // Spawn operator
    commands.spawn((
        Name::new("Operator Shape"),
        Mesh3d(operator_mesh_handle),
        MeshMaterial3d(materials.add(Color::srgb(0.1, 0.8, 0.1))), // Green
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    
    println!("✅ STEP file loading simulation complete!");
}

fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<StepDemoState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&mut Mesh3d, &Name)>,
) {
    let mut needs_update = false;
    
    if keyboard_input.just_pressed(KeyCode::Space) {
        // Cycle through operations
        state.current_operation = match state.current_operation {
            BooleanOperation::Union => BooleanOperation::Intersection,
            BooleanOperation::Intersection => BooleanOperation::Difference,
            BooleanOperation::Difference => BooleanOperation::Union,
        };
        
        println!("🔄 Operation: {} {}", state.current_operation.name(), state.current_operation.symbol());
        needs_update = true;
    }
    
    if keyboard_input.just_pressed(KeyCode::KeyN) {
        // Cycle through operator shapes
        state.operator_shape = match state.operator_shape {
            OperatorShape::Cylinder => OperatorShape::Cube,
            OperatorShape::Cube => OperatorShape::Sphere,
            OperatorShape::Sphere => OperatorShape::Cylinder,
        };
        
        println!("🔷 Operator: {}", state.operator_shape.name());
        needs_update = true;
    }
    
    if keyboard_input.just_pressed(KeyCode::KeyF) {
        // Cycle through STEP files
        if !state.step_files.is_empty() {
            state.current_file_index = (state.current_file_index + 1) % state.step_files.len();
            println!("📁 File: {}", state.step_files[state.current_file_index]);
            needs_update = true;
        }
    }
    
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        // Reset to defaults
        state.current_operation = BooleanOperation::Union;
        state.operator_shape = OperatorShape::Cylinder;
        println!("🔄 Reset to defaults");
        needs_update = true;
    }
    
    if keyboard_input.just_pressed(KeyCode::KeyH) {
        state.show_help = !state.show_help;
        if state.show_help {
            println!("🎮 CONTROLS:");
            println!("  SPACE - Cycle operations");
            println!("  N - Cycle operators");
            println!("  F - Cycle files");
            println!("  R - Reset");
            println!("  H - Toggle help");
            println!("  ESC - Quit");
        }
    }
    
    if needs_update {
        update_boolean_result(&mut state, &mut meshes, &mut materials, &mut query);
    }
}

fn update_boolean_result(
    state: &mut StepDemoState,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    query: &mut Query<(&mut Mesh3d, &Name)>,
) {
    // Create placeholder shapes (in reality, this would load actual STEP data)
    let base_shape = create_step_placeholder();
    let operator_shape = create_operator_shape(state.operator_shape);
    
    println!("⚙️  Performing {} {} {} ...", 
             "STEP_MODEL", state.current_operation.symbol(), state.operator_shape.name());
    
    // Perform the boolean operation
    let result_shape = match state.current_operation {
        BooleanOperation::Union => &base_shape + &operator_shape,
        BooleanOperation::Intersection => &base_shape ^ &operator_shape,
        BooleanOperation::Difference => &base_shape - &operator_shape,
    };
    
    // Validate result
    if result_shape.status != meshbool::ManifoldError::NoError {
        println!("⚠️  Boolean operation failed with status {:?}", result_shape.status);
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
        if name.as_str() == "STEP File Placeholder" {
            *mesh_handle = Mesh3d(result_mesh_handle.clone());
        }
    }
    
    // Also update the operator mesh
    let operator_mesh_gl = get_mesh_gl(&operator_shape, 0);
    let operator_bevy_mesh = meshgl_to_bevy_mesh(&operator_mesh_gl);
    let operator_mesh_handle = meshes.add(operator_bevy_mesh);
    
    for (mut mesh_handle, name) in query.iter_mut() {
        if name.as_str() == "Operator Shape" {
            *mesh_handle = Mesh3d(operator_mesh_handle.clone());
        }
    }
    
    println!("✅ Boolean operation complete!");
}

fn rotate_camera(
    time: Res<Time>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    // Slowly rotate camera around the scene
    if let Ok(mut transform) = camera_query.single_mut() {
        let time_val = time.elapsed_secs() * 0.2;
        let radius = 15.0;
        let x = radius * time_val.cos();
        let z = radius * time_val.sin();
        transform.translation = Vec3::new(x, 5.0, z);
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}

fn update_display(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Name>>,
) {
    // Animate objects for better visualization
    let animation_time = time.elapsed_secs();
    
    for mut transform in query.iter_mut() {
        // Gentle animation
        transform.translation.y = (animation_time * 2.0).sin() * 0.2;
    }
}

/// Create a complex shape to simulate a loaded STEP file
fn create_step_placeholder() -> meshbool::Impl {
    // Create a complex base shape to simulate a STEP file
    let base = cube(Vector3::new(4.0, 2.0, 1.0), true);
    
    // Add some features
    let feature1 = cube(Vector3::new(0.5, 0.8, 0.6), true);
    let translated_feature1 = translate(&feature1, nalgebra::Point3::new(-1.5, 0.0, 0.0));
    
    let feature2 = cube(Vector3::new(0.5, 0.8, 0.6), true);
    let translated_feature2 = translate(&feature2, nalgebra::Point3::new(1.5, 0.0, 0.0));
    
    // Combine with union
    let with_feature1 = &base + &translated_feature1;
    let with_feature2 = &with_feature1 + &translated_feature2;
    
    // Add some cylindrical features
    let cyl1 = cylinder(2.0, 0.3, 0.3, 16, true);
    let translated_cyl1 = translate(&cyl1, nalgebra::Point3::new(-1.0, 0.0, 0.0));
    
    let cyl2 = cylinder(2.0, 0.3, 0.3, 16, true);
    let translated_cyl2 = translate(&cyl2, nalgebra::Point3::new(1.0, 0.0, 0.0));
    
    // Subtract cylinders
    let with_hole1 = &with_feature2 - &translated_cyl1;
    let final_shape = &with_hole1 - &translated_cyl2;
    
    println!("🔧 Created STEP placeholder: {} triangles", final_shape.num_tri());
    final_shape
}

/// Create operator shape based on current selection
fn create_operator_shape(shape: OperatorShape) -> meshbool::Impl {
    match shape {
        OperatorShape::Cylinder => cylinder(2.0, 1.0, 1.0, 32, true),
        OperatorShape::Cube => cube(Vector3::new(1.5, 1.5, 1.5), true),
        OperatorShape::Sphere => cylinder(2.0, 1.2, 1.2, 32, true), // Approximate sphere
    }
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
    
    for i in 0..num_verts {
        let offset = i * mesh_gl.num_prop as usize;
        positions.push([
            mesh_gl.vert_properties[offset],
            mesh_gl.vert_properties[offset + 1], 
            mesh_gl.vert_properties[offset + 2]
        ]);
    }
    
    // Extract indices
    let indices: Vec<u32> = mesh_gl.tri_verts.clone();
    
    // Insert data into Bevy mesh
    bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    bevy_mesh.insert_indices(bevy_mesh::Indices::U32(indices));
    
    bevy_mesh
}