use bevy::prelude::*;
use bevy_mesh_boolean::*;
use bevy_step_loader::*;

#[derive(Component)]
struct StepModel;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            MeshBooleanPlugin,
            StepPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (test_step_boolean_operation, toggle_boolean_op, orbit_camera))
        .run();
}

#[derive(Resource)]
struct StepHandle(Handle<StepAsset>);

#[derive(Resource, Default)]
struct TestState {
    step_loaded: bool,
    operation_performed: bool,
}

#[derive(Resource, Default)]
struct OrbitState {
    angle: f32,
    center: Vec3,
    distance: f32,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Load the STEP file
    eprintln!("[DEBUG] Loading STEP file from assets/multifeature.step");
    let step_handle: Handle<StepAsset> = asset_server.load("multifeature.step");
    commands.insert_resource(StepHandle(step_handle));
    commands.insert_resource(TestState::default());
    commands.insert_resource(OrbitState {
        angle: 0.0,
        center: Vec3::ZERO,
        distance: 10.0,
    });
    
    // Camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 3.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        OrbitCamera,
    ));

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

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 800.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(-4.0, 5.0, -4.0),
        ..default()
    });
}

#[derive(Component)]
struct OrbitCamera;

fn orbit_camera(
    mut query: Query<&mut Transform, With<OrbitCamera>>,
    orbit_state: Res<OrbitState>,
) {
    if let Ok(mut transform) = query.get_single_mut() {
        let x = orbit_state.center.x + orbit_state.distance * orbit_state.angle.cos();
        let z = orbit_state.center.z + orbit_state.distance * orbit_state.angle.sin();
        let y = orbit_state.center.y + 3.0; // Keep a slight elevation
        
        *transform = Transform::from_translation(Vec3::new(x, y, z))
            .looking_at(orbit_state.center, Vec3::Y);
    }
}

fn toggle_boolean_op(
    mut op_state: ResMut<BooleanOpState>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        *op_state = match *op_state {
            BooleanOpState::None => BooleanOpState::Subtract,
            BooleanOpState::Subtract => BooleanOpState::Intersect,
            BooleanOpState::Intersect => BooleanOpState::Union,
            BooleanOpState::Union => BooleanOpState::None,
        };
    }
}

fn test_step_boolean_operation(
    mut commands: Commands,
    step_assets: Res<Assets<StepAsset>>,
    step_handle: Res<StepHandle>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut test_state: ResMut<TestState>,
    mut orbit_state: ResMut<OrbitState>,
) {
    if test_state.operation_performed {
        return;
    }
    
    eprintln!("[TEST] Checking for STEP asset...");
    if let Some(step_asset) = step_assets.get(&step_handle.0) {
        eprintln!("[TEST] STEP asset found!");
        if !test_state.step_loaded {
            eprintln!("[TEST] STEP file loaded successfully!");
            
            // Get mesh statistics
            let vertex_count = if let Some(positions) = step_asset.mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
                match positions {
                    bevy::render::mesh::VertexAttributeValues::Float32x3(pos) => pos.len(),
                    _ => 0,
                }
            } else {
                0
            };
            
            eprintln!("[TEST] STEP mesh has {} vertices", vertex_count);
            test_state.step_loaded = true;
        }
        
        // Perform boolean operation immediately when STEP is loaded
        if !test_state.operation_performed {
            eprintln!("[TEST] Performing boolean operation on STEP file...");
            
            // Spawn the STEP model
            let step_mesh_handle = meshes.add(step_asset.mesh.clone());
            let step_material = materials.add(Color::srgb(0.8, 0.7, 0.6));
            
            let step_entity = commands.spawn((
                PbrBundle {
                    mesh: step_mesh_handle,
                    material: step_material,
                    transform: Transform::from_xyz(-0.5, 0.0, 0.0),
                    ..default()
                },
                StepModel,
            )).id();
            
            eprintln!("[TEST] Spawned STEP model entity: {:?}", step_entity);
            
            // Calculate model center for camera orbit
            if let Some(positions) = step_asset.mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
                if let bevy::render::mesh::VertexAttributeValues::Float32x3(pos) = positions {
                    if !pos.is_empty() {
                        let mut min_bound = Vec3::new(pos[0][0], pos[0][1], pos[0][2]);
                        let mut max_bound = min_bound;
                        
                        for vertex in pos.iter() {
                            let v = Vec3::new(vertex[0], vertex[1], vertex[2]);
                            min_bound = min_bound.min(v);
                            max_bound = max_bound.max(v);
                        }
                        
                        orbit_state.center = (min_bound + max_bound) * 0.5;
                        let model_size = (max_bound - min_bound).length();
                        orbit_state.distance = model_size.max(5.0) * 1.5; // Adjust camera distance based on model size
                    }
                }
            }
            
            // Spawn a cube that will be used for subtraction
            let cube_mesh = meshes.add(Cuboid::from_size(Vec3::splat(20.0))); // Make it large enough to cut
            let cube_material = materials.add(Color::srgb(0.6, 0.8, 0.9));
            
            let cube_entity = commands.spawn(PbrBundle {
                mesh: cube_mesh,
                material: cube_material,
                transform: Transform::from_xyz(0.5, 0.0, 0.0), // Position to intersect with STEP model
                ..default()
            }).id();
            
            eprintln!("[TEST] Spawned cube entity: {:?}", cube_entity);
            
            // Create result entity (hidden initially)
            let result_entity = commands.spawn(PbrBundle {
                material: materials.add(Color::srgb(0.9, 0.5, 0.5)), // Different color for result
                visibility: Visibility::Hidden, // Initially hidden
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..default()
            }).id();
            
            eprintln!("[TEST] Created result entity: {:?}", result_entity);
            
            // Set up the boolean operation handles
            commands.insert_resource(BooleanHandles {
                primary_entity: step_entity,
                secondary_entity: cube_entity,
                result_entity,
            });
            
            eprintln!("[TEST] Inserted BooleanHandles resource");
            
            // Set the boolean operation state to Subtract
            commands.insert_resource(BooleanOpState::Subtract);
            
            test_state.operation_performed = true;
            eprintln!("[TEST] Boolean subtract operation initiated!");
        }
    } else {
        eprintln!("[TEST] STEP asset not yet loaded...");
    }
}