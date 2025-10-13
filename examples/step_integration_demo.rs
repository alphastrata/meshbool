//! MeshBool + STEP File Integration Demo
//! 
//! This demo shows how meshbool would work with STEP files loaded via bevy_step_plugin.
//! It loads a STEP file, then performs boolean operations with primitive shapes.

use bevy::prelude::*;
use meshbool::{cube, cylinder, get_mesh_gl, translate};

// Mock components that would come from bevy_step_plugin
#[derive(Component)]
struct StepFile {
    path: String,
    // In reality, this would contain the loaded mesh data
}

#[derive(Component)]
struct BooleanOperation {
    operation_type: BooleanOpType,
    operand_entity: Entity, // The entity being operated on
}

#[derive(Clone, Copy, PartialEq)]
enum BooleanOpType {
    Union,
    Intersection,
    Difference,
}

impl BooleanOpType {
    fn name(&self) -> &'static str {
        match self {
            BooleanOpType::Union => "UNION",
            BooleanOpType::Intersection => "INTERSECTION",
            BooleanOpType::Difference => "DIFFERENCE",
        }
    }
    
    fn symbol(&self) -> &'static str {
        match self {
            BooleanOpType::Union => "∪",
            BooleanOpType::Intersection => "∩",
            BooleanOpType::Difference => "−",
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // In reality, you'd add your bevy_step_plugin here
        // .add_plugins(StepPlugin)
        .add_systems(Startup, (setup_scene, load_step_file))
        .add_systems(Update, (
            handle_user_input,
            rotate_camera,
            update_boolean_operations
        ))
        .run();
}

#[derive(Resource)]
struct DemoState {
    current_operation: BooleanOpType,
    step_entity: Option<Entity>,
    operator_entity: Option<Entity>,
    show_help: bool,
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    println!("🔧 Setting up STEP file boolean operations demo...");
    
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
    commands.insert_resource(DemoState {
        current_operation: BooleanOpType::Union,
        step_entity: None,
        operator_entity: None,
        show_help: true,
    });
    
    println!("🎮 Controls:");
    println!("  SPACE - Cycle through boolean operations");
    println!("  N - Cycle through operator shapes");
    println!("  R - Reset to default");
    println!("  H - Toggle help");
    println!("  ESC - Quit");
    println!();
    println!("✅ Scene setup complete!");
}

fn load_step_file(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: ResMut<DemoState>,
) {
    println!("📂 Loading STEP file...");
    
    // In reality, this would use your bevy_step_plugin to load the file
    // For demo purposes, I'll simulate loading a STEP file by creating a complex mesh
    
    // Simulate loading a STEP file - in reality this would be:
    // let step_mesh = step_plugin.load_step_file("/path/to/file.step");
    
    // For demo, create a complex shape to simulate a loaded STEP file
    let base_shape = create_complex_step_shape();
    let base_mesh_gl = get_mesh_gl(&base_shape, 0);
    let base_bevy_mesh = meshgl_to_bevy_mesh(&base_mesh_gl);
    let base_mesh_handle = meshes.add(base_bevy_mesh);
    
    println!("✓ Loaded STEP file simulation: {} triangles", base_shape.num_tri());
    
    // Spawn the STEP file entity
    let step_entity = commands.spawn((
        Name::new("STEP Model"),
        StepFile {
            path: "22mm_dovetail_block.step".to_string(),
        },
        Mesh3d(base_mesh_handle),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))), // Silver/gray
        Transform::from_xyz(0.0, 0.0, 0.0),
    )).id();
    
    state.step_entity = Some(step_entity);
    
    // Create an operator shape (cylinder to start)
    let operator_shape = cylinder(2.0, 1.0, 1.0, 32, true);
    let operator_mesh_gl = get_mesh_gl(&operator_shape, 0);
    let operator_bevy_mesh = meshgl_to_bevy_mesh(&operator_mesh_gl);
    let operator_mesh_handle = meshes.add(operator_bevy_mesh);
    
    println!("✓ Created operator shape: {} triangles", operator_shape.num_tri());
    
    // Spawn the operator entity
    let operator_entity = commands.spawn((
        Name::new("Operator Shape"),
        Mesh3d(operator_mesh_handle),
        MeshMaterial3d(materials.add(Color::srgb(0.1, 0.8, 0.1))), // Green
        Transform::from_xyz(0.0, 0.0, 0.0),
    )).id();
    
    state.operator_entity = Some(operator_entity);
    
    println!("✅ STEP file loading complete!");
}

fn handle_user_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<DemoState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&mut Mesh3d, &Name)>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        // Cycle through operations
        state.current_operation = match state.current_operation {
            BooleanOpType::Union => BooleanOpType::Intersection,
            BooleanOpType::Intersection => BooleanOpType::Difference,
            BooleanOpType::Difference => BooleanOpType::Union,
        };
        
        println!("🔄 Operation: {} {}", state.current_operation.name(), state.current_operation.symbol());
        update_boolean_result(&mut state, &mut meshes, &mut materials, &mut query);
    }
    
    if keyboard_input.just_pressed(KeyCode::KeyN) {
        // Cycle through operator shapes
        // This would normally modify the existing operator entity
        println!("🔷 Cycling operator shape...");
        // In a real implementation, this would update the operator mesh
    }
    
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        // Reset to defaults
        state.current_operation = BooleanOpType::Union;
        println!("🔄 Reset to default operation");
        update_boolean_result(&mut state, &mut meshes, &mut materials, &mut query);
    }
    
    if keyboard_input.just_pressed(KeyCode::KeyH) {
        state.show_help = !state.show_help;
        if state.show_help {
            println!("🎮 CONTROLS:");
            println!("  SPACE - Cycle operations");
            println!("  N - Cycle operator shapes");
            println!("  R - Reset");
            println!("  H - Toggle help");
            println!("  ESC - Quit");
        }
    }
}

fn update_boolean_result(
    state: &mut DemoState,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    query: &mut Query<(&mut Mesh3d, &Name)>,
) {
    // In a real implementation, this would:
    // 1. Get the STEP file mesh data
    // 2. Get the operator shape mesh data  
    // 3. Perform the boolean operation using meshbool
    // 4. Convert result back to Bevy mesh
    // 5. Update the result entity
    
    println!("🔄 Performing boolean operation...");
    
    // Simulate the boolean operation
    let base_shape = create_complex_step_shape();
    let operator_shape = cylinder(2.0, 1.0, 1.0, 32, true);
    
    let result_shape = match state.current_operation {
        BooleanOpType::Union => &base_shape + &operator_shape,
        BooleanOpType::Intersection => &base_shape ^ &operator_shape,
        BooleanOpType::Difference => &base_shape - &operator_shape,
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
    
    // Update the result in the scene
    // In reality, this would update a dedicated result entity
    println!("✅ Boolean operation complete!");
}

fn update_boolean_operations(
    mut commands: Commands,
    state: Res<DemoState>,
    meshes: Res<Assets<Mesh>>,
    materials: Res<Assets<StandardMaterial>>,
) {
    // This system would handle the actual boolean operations
    // For demo purposes, we're just showing the concept
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

/// Create a complex shape to simulate a loaded STEP file
fn create_complex_step_shape() -> meshbool::Impl {
    // Create a more complex shape to simulate what a STEP file might contain
    let cube1 = cube(nalgebra::Vector3::new(3.0, 2.0, 1.0), true);
    let cube2 = cube(nalgebra::Vector3::new(1.0, 1.0, 3.0), true);
    let translated_cube2 = translate(&cube2, nalgebra::Point3::new(1.0, 1.0, 0.0));
    
    // Union them to create a more complex base shape
    let complex_shape = &cube1 + &translated_cube2;
    
    // Add some cylinders for complexity
    let cylinder1 = cylinder(2.0, 0.5, 0.5, 16, true);
    let translated_cyl1 = translate(&cylinder1, nalgebra::Point3::new(0.0, 0.0, 1.5));
    
    let result = &complex_shape + &translated_cyl1;
    
    println!("🔧 Created complex STEP simulation shape: {} triangles", result.num_tri());
    result
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