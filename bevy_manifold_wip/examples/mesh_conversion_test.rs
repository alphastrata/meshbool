use bevy::prelude::*;
use bevy_mesh_boolean::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, setup_test)
        .add_systems(Update, test_mesh_conversion)
        .run();
}

fn setup_test(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    eprintln!("[TEST SETUP] Creating test meshes for conversion verification...");
    
    // Create a simple cube mesh
    let cube_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let cube_material = materials.add(Color::srgb(0.8, 0.7, 0.6));
    
    // Create a simple sphere mesh  
    let sphere_mesh = meshes.add(Sphere::new(0.8));
    let sphere_material = materials.add(Color::srgb(0.6, 0.7, 0.8));
    
    // Spawn test entities
    commands.spawn(PbrBundle {
        mesh: cube_mesh,
        material: cube_material,
        transform: Transform::from_xyz(-1.0, 0.0, 0.0),
        ..default()
    });
    
    commands.spawn(PbrBundle {
        mesh: sphere_mesh,
        material: sphere_material,
        transform: Transform::from_xyz(1.0, 0.0, 0.0),
        ..default()
    });
    
    // Add camera and lighting
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 3.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    
    eprintln!("[TEST SETUP] Test entities spawned successfully");
}

fn test_mesh_conversion(
    meshes: Res<Assets<Mesh>>,
    query: Query<&Handle<Mesh>>,
) {
    eprintln!("[TEST] Testing mesh conversion...");
    
    for mesh_handle in query.iter() {
        if let Some(mesh) = meshes.get(mesh_handle) {
            eprintln!("[TEST] Found mesh with handle: {:?}", mesh_handle.id());
            
            // Get mesh statistics
            let vertex_count = if let Some(positions) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
                match positions {
                    bevy::render::mesh::VertexAttributeValues::Float32x3(pos) => pos.len(),
                    _ => 0,
                }
            } else {
                0
            };
            
            let index_count = if let Some(indices) = mesh.indices() {
                match indices {
                    bevy::render::mesh::Indices::U32(indices_vec) => indices_vec.len(),
                    bevy::render::mesh::Indices::U16(indices_vec) => indices_vec.len(),
                }
            } else {
                0
            };
            
            eprintln!("[TEST] Mesh stats - Vertices: {}, Indices: {}", vertex_count, index_count);
            
            // Try to convert to manifold
            let start_time = std::time::Instant::now();
            let manifold_opt = bevy_mesh_to_manifold(mesh);
            let conversion_time = start_time.elapsed();
            
            match manifold_opt {
                Some(manifold) => {
                    let mesh_info = manifold.to_mesh();
                    let result_vertices = mesh_info.vertices().len();
                    let result_indices = mesh_info.indices().len();
                    eprintln!("[TEST SUCCESS] Converted to manifold in {:?} - Result: {} vertices, {} indices", 
                             conversion_time, result_vertices, result_indices);
                },
                None => {
                    eprintln!("[TEST FAILED] Failed to convert mesh to manifold in {:?} - mesh may not be a valid solid", 
                             conversion_time);
                }
            }
        }
    }
    
    eprintln!("[TEST] Mesh conversion test completed");
    std::process::exit(0);
}