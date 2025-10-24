use bevy::prelude::*;
use bevy_mesh_boolean::*;
use bevy_step_loader::*;

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
            MeshBooleanPlugin,
            StepPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (spawn_step_and_perform_subtract, update_ui_text, toggle_boolean_op, orbit_camera))
        .run();
}

#[derive(Resource)]
struct StepHandleResource(Handle<StepAsset>);

#[derive(Resource, Default)]
struct BooleanOperationState {
    step_loaded: bool,
    boolean_performed: bool,
    entities_created: bool,
}

#[derive(Component)]
struct InfoText;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Load the STEP file
    let step_handle: Handle<StepAsset> = asset_server.load("real_parts/multifeature.step");
    commands.insert_resource(StepHandleResource(step_handle));
    commands.insert_resource(BooleanOperationState::default());
    commands.insert_resource(OrbitState {
        angle: 0.0,
        center: Vec3::ZERO,
        distance: 10.0,
    });
    
    // Spawn camera with orbit capability
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        OrbitCamera,
    ));
    
    // Better lighting setup - multiple lights for better illumination
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(10.0, 10.0, 10.0),
        ..default()
    });
    
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 800.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(-10.0, 5.0, -10.0),
        ..default()
    });
    
    // Add UI text to show status
    commands.spawn((
        TextBundle::from_section(
            "Loading STEP file...",
            TextStyle {
                font_size: 20.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
        InfoText,
    ));
    
    // NOTE: Test cube creation moved to spawn_step_and_perform_subtract where meshes/materials are available
}

fn orbit_camera(
    mut query: Query<&mut Transform, With<OrbitCamera>>,
    orbit_state: Res<OrbitState>,
) {
    if let Ok(mut transform) = query.get_single_mut() {
        let x = orbit_state.center.x + orbit_state.distance * orbit_state.angle.cos();
        let z = orbit_state.center.z + orbit_state.distance * orbit_state.angle.sin();
        let y = orbit_state.center.y + 2.0; // Keep a slight elevation
        
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

fn spawn_step_and_perform_subtract(
    mut commands: Commands,
    step_assets: Res<Assets<StepAsset>>,
    step_handle_resource: Res<StepHandleResource>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut boolean_state: ResMut<BooleanOperationState>,
    mut text_query: Query<&mut Text, With<InfoText>>,
    mut orbit_state: ResMut<OrbitState>,
) {
    if boolean_state.boolean_performed {
        return;
    }
    
    if let Some(step_asset) = step_assets.get(&step_handle_resource.0) {
        if !boolean_state.step_loaded {
            // Update UI
            if let Ok(mut text) = text_query.get_single_mut() {
                text.sections[0].value = "STEP file loaded! Setting up boolean operation...".to_string();
            }
            
            eprintln!("STEP file loaded successfully!");
            boolean_state.step_loaded = true;
        }
        
        if !boolean_state.entities_created {
            // Get mesh statistics
            let vertex_count = if let Some(positions) = step_asset.mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
                match positions {
                    bevy::render::mesh::VertexAttributeValues::Float32x3(pos) => pos.len(),
                    _ => 0,
                }
            } else {
                0
            };
            
            eprintln!("STEP mesh has {} vertices", vertex_count);
            
            // Update UI with stats
            if let Ok(mut text) = text_query.get_single_mut() {
                text.sections[0].value = format!("STEP model: {} vertices. Performing boolean subtract...", vertex_count);
            }
            
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
            
            // Spawn the STEP model
            let step_mesh_handle = meshes.add(step_asset.mesh.clone());
            let step_material = materials.add(Color::srgb(0.8, 0.7, 0.6));
            
            let step_entity = commands.spawn((
                PbrBundle {
                    mesh: step_mesh_handle,
                    material: step_material,
                    transform: Transform::from_xyz(0.0, 0.0, 0.0),
                    ..default()
                },
                StepModel,
            )).id();
            
            // Spawn a cube that will be used for subtraction
            let cube_mesh = meshes.add(Cuboid::from_size(Vec3::splat(20.0))); // Make it large enough to cut
            let cube_material = materials.add(Color::srgb(0.6, 0.8, 0.9));
            
            let cube_entity = commands.spawn(PbrBundle {
                mesh: cube_mesh,
                material: cube_material,
                transform: Transform::from_xyz(10.0, 0.0, 0.0), // Position to intersect with STEP model
                ..default()
            }).id();
            
            // Create result entity (hidden initially)
            let result_entity = commands.spawn(PbrBundle {
                material: materials.add(Color::srgb(0.9, 0.5, 0.5)),
                visibility: Visibility::Hidden,
                ..default()
            }).id();
            
            // Set up the boolean operation handles
            commands.insert_resource(BooleanHandles {
                primary_entity: step_entity,
                secondary_entity: cube_entity,
                result_entity,
            });
            
            // Mark the entities as created
            boolean_state.entities_created = true;
            eprintln!("Entities created - STEP model: {:?}, Cube: {:?}, Result: {:?}", step_entity, cube_entity, result_entity);
            
            // Add a simple test cube to verify rendering works
            commands.spawn(PbrBundle {
                mesh: meshes.add(Cuboid::new(0.5, 0.5, 0.5)),
                material: materials.add(Color::srgb(1.0, 0.0, 0.0)), // Red cube
                transform: Transform::from_xyz(0.0, 2.0, 0.0),
                ..default()
            });
            eprintln!("Test cube spawned for rendering verification");
        }
        
        // Perform the boolean subtraction operation immediately
        if boolean_state.entities_created && !boolean_state.boolean_performed {
            // Set the boolean operation state to Subtract
            commands.insert_resource(BooleanOpState::Subtract);
            boolean_state.boolean_performed = true;
            
            // Update UI
            if let Ok(mut text) = text_query.get_single_mut() {
                text.sections[0].value = "Boolean subtract operation initiated!".to_string();
            }
            
            eprintln!("Boolean subtract operation initiated at {:?}", std::time::Instant::now());
        }
    }
}

fn update_ui_text(
    op_state: Res<BooleanOpState>,
    mut text_query: Query<&mut Text, With<InfoText>>,
) {
    if op_state.is_changed() {
        let text = match *op_state {
            BooleanOpState::None => "Boolean operation: None",
            BooleanOpState::Intersect => "Boolean operation: Intersect",
            BooleanOpState::Union => "Boolean operation: Union",
            BooleanOpState::Subtract => "Boolean operation: Subtract (active)",
        };
        
        if let Ok(mut text_component) = text_query.get_single_mut() {
            // Append to existing text with timestamp
            let timestamp = std::time::Instant::now();
            text_component.sections[0].value = format!("{}[{:?}] {}\n", text_component.sections[0].value, timestamp, text);
        }
    }
}