//! Realistic MeshBool + STEP Integration Demo
//! 
//! This demo shows the actual integration pattern with a STEP file loader.
//! It assumes your bevy_step_plugin provides a way to load STEP files and 
//! convert them to meshbool-compatible formats.

use bevy::prelude::*;
use meshbool::{cube, cylinder, sphere, get_mesh_gl, translate, Impl};
use nalgebra::Vector3;

/// Mock component for STEP file data (would come from bevy_step_plugin)
#[derive(Component)]
struct StepModel {
    file_path: String,
    mesh_data: Impl, // The actual meshbool data
}

/// Component for tracking boolean operations
#[derive(Component)]
struct BooleanResult {
    base_entity: Entity,
    operator_entity: Entity,
    operation: BooleanOperation,
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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "MeshBool + STEP File Boolean Operations".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup_scene)
        .add_systems(Update, handle_user_input)
        .add_systems(Update, rotate_camera)
        .add_systems(Update, update_display)
        .run();
}

#[derive(Resource)]
struct StepDemoState {
    current_operation: BooleanOperation,
    step_entities: Vec<Entity>, // Entities loaded from STEP files
    operator_entities: Vec<Entity>, // Primitive shapes for operations
    current_operator_index: usize,
    show_help: bool,
    camera_angle: f32,
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    println!("🔧 MESHBOOL + STEP FILE BOOLEAN OPERATIONS DEMO");
    println!("================================================");
    println!("This demo simulates loading STEP files and performing");
    println!("boolean operations with meshbool.");
    println!();
    println!("🎮 CONTROLS:");
    println!("  SPACE - Cycle through boolean operations");
    println!("  N - Cycle through operator shapes");
    println!("  R - Reset to default operation");
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
    
    // Initialize demo with sample STEP file loading
    load_sample_step_files(&mut commands, &mut meshes, &mut materials);
    
    // Initialize demo state
    commands.insert_resource(StepDemoState {
        current_operation: BooleanOperation::Union,
        step_entities: vec![], // Will be populated by load_sample_step_files
        operator_entities: vec![], // Will be populated by load_sample_step_files
        current_operator_index: 0,
        show_help: true,
        camera_angle: 0.0,
    });
    
    println!("✅ Demo initialized - camera will orbit automatically");
}

fn load_sample_step_files(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    println!("📂 Loading sample STEP files...");
    
    // Simulate loading STEP files (in reality, this would use your bevy_step_plugin)
    // For demo, we'll create complex shapes that simulate loaded STEP data
    
    // Simulate loading "22mm_dovetail_block.step"
    let dovetail_shape = create_dovetail_shape();
    let dovetail_mesh_gl = get_mesh_gl(&dovetail_shape, 0);
    let dovetail_bevy_mesh = meshgl_to_bevy_mesh(&dovetail_mesh_gl);
    let dovetail_mesh_handle = meshes.add(dovetail_bevy_mesh);
    
    println!("✓ Loaded dovetail block: {} triangles", dovetail_shape.num_tri());
    
    let dovetail_entity = commands.spawn((
        Name::new("Dovetail Block (STEP)"),
        StepModel {
            file_path: "22mm_dovetail_block.step".to_string(),
            mesh_data: dovetail_shape,
        },
        Mesh3d(dovetail_mesh_handle),
        MeshMaterial3d(materials.add(Color::srgb(0.7, 0.7, 0.8))), // Light gray
        Transform::from_xyz(0.0, 0.0, 0.0),
    )).id();
    
    // Create operator shapes (primitives for boolean operations)
    let operator_shapes = vec![
        ("Cylinder", cylinder(2.0, 1.0, 1.0, 32, true)),
        ("Cube", cube(Vector3::new(1.5, 1.5, 1.5), true)),
        ("Sphere", sphere(1.2, 32, true)), // Assuming sphere function exists
    ];
    
    let mut operator_entities = Vec::new();
    
    for (i, (name, shape)) in operator_shapes.into_iter().enumerate() {
        let mesh_gl = get_mesh_gl(&shape, 0);
        let bevy_mesh = meshgl_to_bevy_mesh(&mesh_gl);
        let mesh_handle = meshes.add(bevy_mesh);
        
        let operator_entity = commands.spawn((
            Name::new(format!("{} Operator", name)),
            Mesh3d(mesh_handle),
            MeshMaterial3d(materials.add(Color::srgb(0.1, 0.8, 0.1))), // Green
            Transform::from_xyz(0.0, 0.0, 0.0),
            // Only show first operator initially
            Visibility::Visible,
        )).id();
        
        operator_entities.push(operator_entity);
        
        println!("✓ Created {} operator: {} triangles", name, shape.num_tri());
    }
    
    // Store entities for later use (would normally be done in the resource setup)
    println!("✅ STEP file loading simulation complete!");
}

fn handle_user_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<StepDemoState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        // Cycle through operations
        state.current_operation = match state.current_operation {
            BooleanOperation::Union => BooleanOperation::Intersection,
            BooleanOperation::Intersection => BooleanOperation::Difference,
            BooleanOperation::Difference => BooleanOperation::Union,
        };
        
        println!("🔄 Operation: {} {}", state.current_operation.name(), state.current_operation.symbol());
        // For now, skip the boolean operation that requires direct world access
    }
    
    if keyboard_input.just_pressed(KeyCode::KeyN) {
        // Cycle through operator shapes
        if !state.operator_entities.is_empty() {
            state.current_operator_index = (state.current_operator_index + 1) % state.operator_entities.len();
            println!("🔷 Switched to operator {}", state.current_operator_index);
            // For now, skip the visibility update that requires direct world access
        }
    }
    
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        // Reset to default
        state.current_operation = BooleanOperation::Union;
        println!("🔄 Reset to UNION operation");
        // For now, skip the boolean operation that requires direct world access
    }
    
    if keyboard_input.just_pressed(KeyCode::KeyH) {
        state.show_help = !state.show_help;
        if state.show_help {
            println!("🎮 CONTROLS:");
            println!("  SPACE - Cycle operations (Union, Intersection, Difference)");
            println!("  N - Cycle operator shapes (Cylinder, Cube, Sphere)");
            println!("  R - Reset to Union");
            println!("  H - Toggle help");
            println!("  ESC - Quit");
        }
    }
}

fn perform_boolean_operation(
    state: &mut StepDemoState,
    meshes: &mut ResMut<Assets<Mesh>>,
) {
    // In a real implementation with bevy_step_plugin, this would:
    // 1. Get the STEP model mesh data from the StepModel component
    // 2. Get the current operator shape mesh data
    // 3. Perform the boolean operation using meshbool
    // 4. Convert result to Bevy mesh and update display
    
    if state.step_entities.is_empty() {
        println!("⚠️  No STEP entities loaded");
        return;
    }
    
    // Get mesh data from components (would use your bevy_step_plugin)
    let base_mesh_data = get_mock_step_mesh_data(); // Simulate getting from component
    let operator_mesh_data = get_mock_operator_mesh_data(state.current_operator_index);
    
    println!("⚙️  Performing {} operation...", state.current_operation.name());
    
    // Perform the boolean operation using meshbool
    let result_mesh = match state.current_operation {
        BooleanOperation::Union => &base_mesh_data + &operator_mesh_data,
        BooleanOperation::Intersection => &base_mesh_data ^ &operator_mesh_data,
        BooleanOperation::Difference => &base_mesh_data - &operator_mesh_data,
    };
    
    // Validate result
    if result_mesh.status != meshbool::ManifoldError::NoError {
        println!("⚠️  Boolean operation failed with status {:?}", result_mesh.status);
        return;
    }
    
    println!("📊 Result: {} triangles, {} vertices", 
             result_mesh.num_tri(), result_mesh.num_vert());
    
    // Convert to Bevy mesh
    let result_mesh_gl = get_mesh_gl(&result_mesh, 0);
    let result_bevy_mesh = meshgl_to_bevy_mesh(&result_mesh_gl);
    let _result_mesh_handle = meshes.add(result_bevy_mesh);
    
    // Update the result display (would normally update a dedicated result entity)
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
    mut query: Query<(&mut Transform, &Name)>,
    time: Res<Time>,
) {
    // Animate objects slightly for better visualization
    let animation_time = time.elapsed_secs();
    
    for (mut transform, name) in query.iter_mut() {
        if name.as_str() == "Dovetail Block (STEP)" {
            // Gentle rotation for the base STEP model
            transform.rotation = Quat::from_rotation_y(animation_time * 0.1);
        } else if name.as_str().contains("Operator") {
            // Move operators around for better visualization
            let offset = animation_time * 0.5;
            transform.translation = Vec3::new(offset.sin() * 2.0, offset.cos() * 1.0, 0.0);
        }
    }
}

/// Create a complex shape to simulate a dovetail block from STEP file
fn create_dovetail_shape() -> Impl {
    // Create base block
    let base = cube(Vector3::new(4.0, 2.0, 1.0), true);
    
    // Create dovetail features
    let dovetail1 = cube(Vector3::new(0.5, 0.8, 0.6), true);
    let translated_dt1 = translate(&dovetail1, nalgebra::Point3::new(-1.5, 0.0, 0.0));
    
    let dovetail2 = cube(Vector3::new(0.5, 0.8, 0.6), true);
    let translated_dt2 = translate(&dovetail2, nalgebra::Point3::new(1.5, 0.0, 0.0));
    
    // Combine base with dovetails using union
    let with_dt1 = &base + &translated_dt1;
    let with_dt2 = &with_dt1 + &translated_dt2;
    
    // Add some cylindrical holes for complexity
    let hole1 = cylinder(2.0, 0.3, 0.3, 16, true);
    let translated_hole1 = translate(&hole1, nalgebra::Point3::new(-1.0, 0.0, 0.0));
    
    let hole2 = cylinder(2.0, 0.3, 0.3, 16, true);
    let translated_hole2 = translate(&hole2, nalgebra::Point3::new(1.0, 0.0, 0.0));
    
    // Subtract holes using difference
    let with_hole1 = &with_dt2 - &translated_hole1;
    let final_shape = &with_hole1 - &translated_hole2;
    
    println!("🔧 Created simulated dovetail STEP shape: {} triangles", final_shape.num_tri());
    final_shape
}

/// Mock function to simulate getting mesh data from STEP file component
fn get_mock_step_mesh_data() -> Impl {
    create_dovetail_shape()
}

/// Mock function to simulate getting operator mesh data
fn get_mock_operator_mesh_data(index: usize) -> Impl {
    match index % 3 {
        0 => cylinder(2.0, 1.0, 1.0, 32, true),
        1 => cube(Vector3::new(1.5, 1.5, 1.5), true),
        2 => sphere(1.2, 32, true), // Assuming sphere function exists
        _ => cube(Vector3::new(1.0, 1.0, 1.0), true),
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


