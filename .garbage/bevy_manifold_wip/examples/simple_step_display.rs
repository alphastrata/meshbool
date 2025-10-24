use bevy::prelude::*;
use bevy_step_loader::{StepAsset, StepPlugin};

#[derive(Component)]
struct StepModel;

#[derive(Resource, Default)]
struct OrbitState {
    angle: f32,
    center: Vec3,
    distance: f32,
}

#[derive(Component)]
struct OrbitCamera;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            StepPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (spawn_step_mesh, setup_camera_position, orbit_camera))
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Load the STEP file
    let step_handle: Handle<StepAsset> = asset_server.load("multifeature.step");
    
    // Store the handle to spawn the mesh when it loads
    commands.insert_resource(StepHandleResource(step_handle));
    commands.insert_resource(CameraSetupState::default());
    commands.insert_resource(OrbitState {
        angle: 0.0,
        center: Vec3::ZERO,
        distance: 10.0,
    });
    
    // Lighting
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // Spawn camera (will be positioned later when mesh loads)
    commands.spawn((
        Camera3dBundle {
            projection: Projection::Perspective(PerspectiveProjection {
                fov: std::f32::consts::PI / 4.0,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 1.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        OrbitCamera,
    ));
    
    // Add a simple UI text to show status
    commands.spawn(TextBundle {
        text: Text::from_section(
            "Loading STEP file...",
            TextStyle {
                font_size: 30.0,
                color: Color::WHITE,
                ..default()
            },
        ),
        ..default()
    });
}

fn orbit_camera(
    mut query: Query<&mut Transform, With<OrbitCamera>>,
    mut orbit_state: ResMut<OrbitState>,
) {
    orbit_state.angle += 0.005; // Slowly rotate the camera
    if let Ok(mut transform) = query.get_single_mut() {
        let x = orbit_state.center.x + orbit_state.distance * orbit_state.angle.cos();
        let z = orbit_state.center.z + orbit_state.distance * orbit_state.angle.sin();
        let y = orbit_state.center.y + 2.0; // Keep a slight elevation
        
        *transform = Transform::from_translation(Vec3::new(x, y, z))
            .looking_at(orbit_state.center, Vec3::Y);
    }
}

#[derive(Resource)]
struct StepHandleResource(Handle<StepAsset>);

#[derive(Resource, Default)]
struct CameraSetupState {
    mesh_spawned: bool,
    camera_positioned: bool,
}

#[derive(Component)]
struct OrthoCamera;

fn spawn_step_mesh(
    mut commands: Commands,
    step_assets: Res<Assets<StepAsset>>,
    step_handle_resource: Res<StepHandleResource>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut camera_state: ResMut<CameraSetupState>,
    mut text_query: Query<&mut Text>,
    mut orbit_state: ResMut<OrbitState>,
) {
    if camera_state.mesh_spawned {
        return;
    }
    
    if let Some(step_asset) = step_assets.get(&step_handle_resource.0) {
        eprintln!("STEP file loaded! Spawning mesh...");
        
        // Update UI text
        if let Ok(mut text) = text_query.get_single_mut() {
            text.sections[0].value = "STEP file loaded! Processing...".to_string();
        }
        
        // Get mesh statistics
        let vertex_count = if let Some(positions) = step_asset.mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            match positions {
                bevy::render::mesh::VertexAttributeValues::Float32x3(pos) => pos.len(),
                _ => 0,
            }
        } else {
            0
        };
        
        let triangle_count = if let Some(indices) = step_asset.mesh.indices() {
            match indices {
                bevy::render::mesh::Indices::U32(indices_vec) => indices_vec.len() / 3,
                bevy::render::mesh::Indices::U16(indices_vec) => indices_vec.len() / 3,
            }
        } else {
            0
        };
        
        eprintln!("STEP Mesh Stats - Vertices: {}, Triangles: {}", vertex_count, triangle_count);
        
        // Calculate bounding box for camera positioning
        let (min_bounds, max_bounds) = calculate_mesh_bounds(&step_asset.mesh);
        let mesh_size = max_bounds - min_bounds;
        let mesh_center = (min_bounds + max_bounds) * 0.5;
        
        eprintln!("Mesh bounds: min={:?}, max={:?}", min_bounds, max_bounds);
        eprintln!("Mesh center: {:?}", mesh_center);
        eprintln!("Mesh size: {:?}", mesh_size);
        
        // Update orbit state with mesh info
        orbit_state.center = mesh_center;
        let max_dimension = mesh_size.x.max(mesh_size.y).max(mesh_size.z);
        orbit_state.distance = max_dimension * 1.5;
        
        // Update UI text with stats
        if let Ok(mut text) = text_query.get_single_mut() {
            text.sections[0].value = format!("Vertices: {}, Triangles: {}", vertex_count, triangle_count);
        }
        
        // Spawn the STEP mesh
        let mesh_handle = meshes.add(step_asset.mesh.clone());
        let material_handle = materials.add(Color::srgb(0.8, 0.7, 0.6));
        
        commands.spawn((
            PbrBundle {
                mesh: mesh_handle,
                material: material_handle,
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..default()
            },
            StepModel,
        ));
        
        // Store mesh info for camera setup
        commands.insert_resource(MeshInfo {
            center: mesh_center,
            size: mesh_size,
        });
        
        camera_state.mesh_spawned = true;
        eprintln!("STEP mesh spawned successfully!");
    }
}

#[derive(Resource)]
struct MeshInfo {
    center: Vec3,
    size: Vec3,
}

fn setup_camera_position(
    mut camera_query: Query<&mut Transform, With<OrbitCamera>>,
    mesh_info: Option<Res<MeshInfo>>,
    mut camera_state: ResMut<CameraSetupState>,
    mut orbit_state: ResMut<OrbitState>,
) {
    if camera_state.camera_positioned || mesh_info.is_none() {
        return;
    }
    
    let mesh_info = mesh_info.unwrap();
    
    // Update orbit state with the mesh information
    orbit_state.center = mesh_info.center;
    let max_dimension = mesh_info.size.x.max(mesh_info.size.y).max(mesh_info.size.z);
    orbit_state.distance = max_dimension * 1.5;
    
    eprintln!("Orbit state updated with mesh info");
    eprintln!("Orbit center: {:?}", orbit_state.center);
    eprintln!("Orbit distance: {}", orbit_state.distance);
    
    camera_state.camera_positioned = true;
}

fn calculate_mesh_bounds(mesh: &Mesh) -> (Vec3, Vec3) {
    if let Some(positions) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        match positions {
            bevy::render::mesh::VertexAttributeValues::Float32x3(pos) => {
                if pos.is_empty() {
                    return (Vec3::ZERO, Vec3::ZERO);
                }
                
                let mut min_bound = Vec3::new(pos[0][0], pos[0][1], pos[0][2]);
                let mut max_bound = min_bound;
                
                for vertex in pos.iter() {
                    let v = Vec3::new(vertex[0], vertex[1], vertex[2]);
                    min_bound = min_bound.min(v);
                    max_bound = max_bound.max(v);
                }
                
                (min_bound, max_bound)
            },
            _ => (Vec3::ZERO, Vec3::ZERO),
        }
    } else {
        (Vec3::ZERO, Vec3::ZERO)
    }
}