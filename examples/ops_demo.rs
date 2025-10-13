//! Boolean Operations Demo
//! 
//! This example demonstrates basic boolean operations (union, intersection, difference) 
//! by showing three shapes arranged like an equation: LHS op RHS = OUTPUT

use bevy::prelude::*;
use meshbool::{cube, cylinder, get_mesh_gl, translate};
use nalgebra::Vector3;

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

    // Create LHS shape (the "victim") - a cube with some features
    let lhs_shape = create_lhs_shape();
    let lhs_mesh_gl = get_mesh_gl(&lhs_shape, 0);
    let lhs_bevy_mesh = meshgl_to_bevy_mesh(&lhs_mesh_gl);
    let lhs_mesh_handle = meshes.add(lhs_bevy_mesh);

    // Create RHS shape (the "operator") - a cylinder
    let rhs_shape = cylinder(2.0, 0.8, 0.8, 32, true);
    let rhs_mesh_gl = get_mesh_gl(&rhs_shape, 0);
    let rhs_bevy_mesh = meshgl_to_bevy_mesh(&rhs_mesh_gl);
    let rhs_mesh_handle = meshes.add(rhs_bevy_mesh);

    // Create the initial output shape (result of the operation)
    let output_shape = match OperationType::ViewOriginal {
        OperationType::ViewOriginal => lhs_shape.clone(),
        OperationType::BooleanUnion => &lhs_shape + &rhs_shape,
        OperationType::BooleanIntersection => &lhs_shape ^ &rhs_shape,
        OperationType::BooleanDifference => &lhs_shape - &rhs_shape,
    };
    let output_mesh_gl = get_mesh_gl(&output_shape, 0);
    let output_bevy_mesh = meshgl_to_bevy_mesh(&output_mesh_gl);
    let output_mesh_handle = meshes.add(output_bevy_mesh);

    // Create materials for the three shapes
    let lhs_material = materials.add(Color::srgb(0.7, 0.7, 0.8));  // Light gray
    let rhs_material = materials.add(Color::srgb(0.1, 0.8, 0.1));  // Green
    let output_material = materials.add(Color::srgb(0.8, 0.5, 0.2)); // Orange

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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&mut Mesh3d, &Name)>,
) {
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
    state: &mut DemoState,
    meshes: &mut ResMut<Assets<Mesh>>,
    _materials: &mut ResMut<Assets<StandardMaterial>>,
    query: &mut Query<(&mut Mesh3d, &Name)>,
) {
    // Create LHS shape (the "victim")
    let lhs_shape = create_lhs_shape();

    // Create RHS shape (the "operator")
    let rhs_shape = cylinder(2.0, 0.8, 0.8, 32, true);

    // Perform the selected operation
    let result_shape = match state.current_operation {
        OperationType::ViewOriginal => lhs_shape.clone(), // Just show the original lhs shape
        OperationType::BooleanUnion => &lhs_shape + &rhs_shape,
        OperationType::BooleanIntersection => &lhs_shape ^ &rhs_shape,
        OperationType::BooleanDifference => &lhs_shape - &rhs_shape,
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

    // Update only the output/result mesh in the center
    for (mut mesh_handle, name) in query.iter_mut() {
        if name.as_str() == "Output Shape (Result)" {
            *mesh_handle = Mesh3d(result_mesh_handle.clone());
        }
    }
}

/// Create a complex LHS shape to serve as the "victim"
fn create_lhs_shape() -> meshbool::Impl {
    // Create a base cube
    let base = cube(Vector3::new(2.0, 2.0, 2.0), true);

    // Add some features to make it more interesting
    let feature1 = cube(Vector3::new(0.8, 1.0, 1.0), true);
    let translated_feature1 = translate(&feature1, nalgebra::Point3::new(-1.0, 0.0, 0.0));

    let feature2 = cube(Vector3::new(0.8, 1.0, 1.0), true);
    let translated_feature2 = translate(&feature2, nalgebra::Point3::new(1.0, 0.0, 0.0));

    // Combine with unions
    let with_features = &base + &translated_feature1;
    let final_shape = &with_features + &translated_feature2;

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