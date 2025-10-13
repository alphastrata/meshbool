use bevy::prelude::*;
use meshbool::{cube, cylinder, get_mesh_gl};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "MeshBool Boolean Operations Demo".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_input, rotate_camera))
        .run();
}

#[derive(Resource)]
struct DemoState {
    base_shape: meshbool::Impl,
    operator_shape: meshbool::Impl,
    current_operation: Operation,
    show_help: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum Operation {
    BaseOnly,
    Union,
    Intersection, 
    Difference,
}

impl Operation {
    fn name(&self) -> &'static str {
        match self {
            Operation::BaseOnly => "BASE SHAPE ONLY",
            Operation::Union => "BOOLEAN UNION (A ∪ B)",
            Operation::Intersection => "BOOLEAN INTERSECTION (A ∩ B)",
            Operation::Difference => "BOOLEAN DIFFERENCE (A − B)",
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    println!("🎮 MESHBOOL BOOLEAN OPERATIONS DEMO");
    println!("===================================");
    println!("Controls:");
    println!("  SPACE - Cycle through operations");
    println!("  H - Toggle help text");
    println!("  ESC - Quit");
    println!();
    
    // Create base shape (cube) and operator shape (cylinder)
    let base_shape = cube(nalgebra::Vector3::new(2.0, 2.0, 2.0), true);
    let operator_shape = cylinder(2.0, 1.0, 1.0, 32, true);
    
    println!("✓ Base shape: Cube ({} triangles)", base_shape.num_tri());
    println!("✓ Operator shape: Cylinder ({} triangles)", operator_shape.num_tri());
    
    // Convert shapes to Bevy meshes
    let base_mesh_gl = get_mesh_gl(&base_shape, 0);
    let base_bevy_mesh = meshgl_to_bevy_mesh(&base_mesh_gl);
    let base_mesh_handle = meshes.add(base_bevy_mesh);
    
    let operator_mesh_gl = get_mesh_gl(&operator_shape, 0);
    let operator_bevy_mesh = meshgl_to_bevy_mesh(&operator_mesh_gl);
    let operator_mesh_handle = meshes.add(operator_bevy_mesh);
    
    // Create initial result (base shape only)
    let result_mesh_gl = get_mesh_gl(&base_shape, 0);
    let result_bevy_mesh = meshgl_to_bevy_mesh(&result_mesh_gl);
    let result_mesh_handle = meshes.add(result_bevy_mesh);
    
    // Store demo state
    commands.insert_resource(DemoState {
        base_shape,
        operator_shape,
        current_operation: Operation::BaseOnly,
        show_help: true,
    });
    
    // Spawn result mesh (what we see)
    commands.spawn((
        Name::new("Result"),
        Mesh3d(result_mesh_handle),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.1, 0.1))), // Red
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    
    // Spawn base shape for reference
    commands.spawn((
        Name::new("Base Reference"),
        Mesh3d(base_mesh_handle),
        MeshMaterial3d(materials.add(Color::srgba(0.1, 0.1, 0.8, 0.3))), // Semi-transparent blue
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    
    // Spawn operator shape for reference
    commands.spawn((
        Name::new("Operator Reference"),
        Mesh3d(operator_mesh_handle),
        MeshMaterial3d(materials.add(Color::srgba(0.1, 0.8, 0.1, 0.5))), // Semi-transparent green
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    
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
        Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    
    println!("✅ Demo initialized!");
    println!("   Press SPACE to cycle operations");
}

fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<DemoState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(&mut Mesh3d, &Name)>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        // Cycle through operations
        state.current_operation = match state.current_operation {
            Operation::BaseOnly => Operation::Union,
            Operation::Union => Operation::Intersection,
            Operation::Intersection => Operation::Difference,
            Operation::Difference => Operation::BaseOnly,
        };
        
        println!("🔄 {}", state.current_operation.name());
        
        // Perform the boolean operation
        let result_shape = match state.current_operation {
            Operation::BaseOnly => state.base_shape.clone(),
            Operation::Union => &state.base_shape + &state.operator_shape,
            Operation::Intersection => &state.base_shape ^ &state.operator_shape,
            Operation::Difference => &state.base_shape - &state.operator_shape,
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
        
        // Update the result mesh in the scene
        for (mut mesh_handle, name) in query.iter_mut() {
            if name.as_str() == "Result" {
                *mesh_handle = Mesh3d(meshes.add(result_bevy_mesh.clone()));
            }
        }
    }
    
    if keyboard_input.just_pressed(KeyCode::KeyH) {
        state.show_help = !state.show_help;
        if state.show_help {
            println!("🎮 CONTROLS:");
            println!("  SPACE - Cycle operations");
            println!("  H - Toggle help");
            println!("  ESC - Quit");
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
        let radius = 10.0;
        let x = radius * time_val.cos();
        let z = radius * time_val.sin();
        transform.translation = Vec3::new(x, 3.0, z);
        transform.look_at(Vec3::ZERO, Vec3::Y);
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