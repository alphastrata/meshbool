//! Simple demo that loads a STEP file and shows all three meshes simultaneously
//! LHS (primary), RHS (secondary), and Result of boolean operation
//! With space bar toggle and camera orbiting

use bevy::{
    core_pipeline::tonemapping::Tonemapping,
    prelude::*,
};
use bevy_mesh_boolean::*;
use bevy_step_loader::*;
use std::path::PathBuf;
use bevy::log;

// Parse command line arguments to get the STEP file path
fn parse_cli_args() -> CliArgs {
    let args: Vec<String> = std::env::args().collect();
    
    // Default to None for the STEP file path, which means we'll use the fallback
    let mut step_file_path = None;
    let mut initial_boolean_op = BooleanOpState::None; // Default to None to show original shapes first
    
    // Parse arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--step" | "-s" => {
                if i + 1 < args.len() {
                    step_file_path = Some(PathBuf::from(&args[i + 1]));
                    i += 1; // Skip the next argument as it's the value
                } else {
                    eprintln!("Error: --step/-s requires a path argument");
                    std::process::exit(1);
                }
            }
            "--op" | "-o" => {
                if i + 1 < args.len() {
                    let op_str = &args[i + 1].to_lowercase();
                    initial_boolean_op = match op_str.as_str() {
                        "none" => BooleanOpState::None,
                        "intersect" | "intersection" => BooleanOpState::Intersect,
                        "union" => BooleanOpState::Union,
                        "subtract" | "difference" => BooleanOpState::Subtract,
                        _ => {
                            eprintln!("Error: Invalid operation '{}'. Valid options: none, intersect, union, subtract", op_str);
                            std::process::exit(1);
                        }
                    };
                    i += 1; // Skip the next argument as it's the value
                } else {
                    eprintln!("Error: --op/-o requires an operation argument");
                    std::process::exit(1);
                }
            }
            path if path.ends_with(".step") || path.ends_with(".stp") => {
                step_file_path = Some(PathBuf::from(path));
            }
            _ => {
                eprintln!("Error: Unknown argument '{}'", args[i]);
                eprintln!("Usage: {} [STEP_FILE_PATH | --step PATH] [--op OPERATION]", &args[0]);
                std::process::exit(1);
            }
        }
        i += 1;
    }
    
    CliArgs {
        step_file: step_file_path,
        initial_boolean_op,
    }
}

fn main() {
    
    let cli_args = parse_cli_args();
    
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            MeshBooleanPlugin,
            StepPlugin,
        ))
        .insert_resource(ClearColor(Color::srgb(0.15, 0.15, 0.15)))
        .insert_resource(AmbientLight {
            brightness: 0.75,
            color: Color::WHITE,
        })
        .insert_resource(cli_args)
        .insert_resource(SecondaryShape::Cube) // Initialize with Cube as the default shape
        .insert_resource(GeometryStats::default())
        .insert_resource(OrbitState::default())
        .add_systems(Startup, (setup, setup_ui, setup_aluminum_material))
        .add_systems(Update, (
            load_step_and_setup_meshes,
            cycle_boolean_op,
            cycle_secondary_shape,
            update_operation_text,
            update_stats_text,
            orbit_camera,
            update_orbit_state,
            exit_on_q_key,
            ensure_aluminum_material,
        ).chain())
        .run();
}

// System to create and store the aluminum material
fn setup_aluminum_material(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let aluminum_material = StandardMaterial {
        base_color: Color::srgb(0.8, 0.8, 0.8), // Light gray
        metallic: 0.95, // High metallic value for aluminum-like appearance
        perceptual_roughness: 0.1, // Low roughness for shiny surface
        reflectance: 0.9, // High reflectance
        ..default()
    };
    
    let aluminum_handle = materials.add(aluminum_material);
    commands.insert_resource(AluminumMaterialHandle(aluminum_handle));
}

// System to track the aluminum material handle
#[derive(Resource)]
struct AluminumMaterialHandle(Handle<StandardMaterial>);

// System to ensure the STEP model always has the aluminum material
fn ensure_aluminum_material(
    mut commands: Commands,
    step_query: Query<(Entity, &Handle<StandardMaterial>), (With<StepModel>, Without<SecondaryBooleanMesh>, Without<ResultShape>)>,
    aluminum_material: Option<Res<AluminumMaterialHandle>>,
    materials: Res<Assets<StandardMaterial>>,
) {
    if let Some(aluminum_handle) = aluminum_material {
        // Apply aluminum material to STEP model if it doesn't have it
        for (entity, current_material) in step_query.iter() {
            // Check if current material is not the aluminum material by checking some properties
            if let Some(material) = materials.get(current_material) {
                // If it's already aluminum-like, continue
                if (material.metallic - 0.95).abs() < 0.01 && (material.perceptual_roughness - 0.1).abs() < 0.01 {
                    continue;
                }
            }
            
            // Update the entity with aluminum material
            commands.entity(entity).insert(aluminum_handle.0.clone());
        }
    }
}

#[derive(Component)]
struct ResultShape;

#[derive(Resource)]
struct OrbitState {
    angle: f32,
    center: Vec3,
    distance: f32,
}

impl Default for OrbitState {
    fn default() -> Self {
        OrbitState {
            angle: 0.0,
            center: Vec3::new(0.0, 0.0, 0.0), // Initially at origin, will be updated with STEP model center
            distance: 150.0, // Initial distance, will be updated based on STEP model size
        }
    }
}

#[derive(Component)]
struct OrbitCamera;

// A marker component for the operation text UI element
#[derive(Component)]
struct OperationText;

// A marker component for the statistics text UI element
#[derive(Component)]
struct StatsText;

// A resource to hold the current primitive type for the secondary shape
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
enum SecondaryShape {
    Cube,
    Sphere,
}

impl Default for SecondaryShape {
    fn default() -> Self {
        SecondaryShape::Cube
    }
}

// A resource to hold current geometry statistics
#[derive(Resource, Default, Debug, Clone, Copy)]
struct GeometryStats {
    vertices: usize,
    edges: usize,
}

// A resource to hold the original scaling information for the secondary shape
#[derive(Resource, Debug, Clone, Copy)]
struct SecondaryShapeScale {
    original_scale: f32,
}

impl Default for SecondaryShapeScale {
    fn default() -> Self {
        SecondaryShapeScale {
            original_scale: 1.0, // Default to unit scale
        }
    }
}

// A resource to hold current timing information
#[derive(Resource, Default, Debug, Clone, Copy)]
struct TimingInfo {
    total_time: std::time::Duration,
    mesh_conversion: std::time::Duration,
    transform: std::time::Duration,
    boolean_op: std::time::Duration,
    mesh_conversion_back: std::time::Duration,
    update_entity: std::time::Duration,
}

// A resource to hold the handles of the entities and meshes used in the demo
#[derive(Resource)]
struct DemoHandles {
    step_shape: Entity,
    secondary_shape: Entity,
    result: Entity,
    original_step_mesh: Handle<Mesh>,
    original_secondary_mesh: Handle<Mesh>,
}

// A resource to hold the CLI arguments
#[derive(Resource, Debug)]
struct CliArgs {
    step_file: Option<PathBuf>,
    initial_boolean_op: BooleanOpState,
}

fn setup(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    _meshes: ResMut<Assets<Mesh>>,
    _materials: ResMut<Assets<StandardMaterial>>,
) {
    debug!("Setting up the scene");

    // Add multiple directional lights for better illumination of the model
    // Main light from above and slightly to the side
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 3000.0,  // Increased illuminance for better lighting
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::YXZ,
            45.0_f32.to_radians(),
            -45.0_f32.to_radians(),
            0.0,
        )),
        ..default()
    });

    // Secondary light from a different angle to fill shadows
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 1500.0,  // Increased illuminance
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::YXZ,
            -135.0_f32.to_radians(),
            30.0_f32.to_radians(),
            0.0,
        )),
        ..default()
    });

    // Tertiary light from below to reduce harsh shadows underneath
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 1000.0,
            shadows_enabled: true,
            color: Color::Srgba(Srgba::new(0.9, 0.9, 1.0, 1.0)), // Slightly blue-white for ambient light
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::YXZ,
            0.0_f32.to_radians(),
            75.0_f32.to_radians(),
            0.0,
        )),
        ..default()
    });

    // Position camera to look at the origin with appropriate distance
    // The final position will be updated when the STEP model is loaded
    let cam_trans =
        Transform::from_xyz(0.0, 50.0, 150.0)
            .looking_at(Vec3::ZERO, Vec3::Y);

    // Spawn camera with an attached light for better illumination of what we're looking at
    let camera_entity = commands.spawn((
        Camera3dBundle {
            transform: cam_trans,
            tonemapping: Tonemapping::AcesFitted,
            ..default()
        },
        OrbitCamera,
    )).id();

    // Add a spotlight attached to the camera to illuminate what we're looking at
    commands.entity(camera_entity).with_children(|parent| {
        parent.spawn(PointLightBundle {
            point_light: PointLight {
                intensity: 3000.0,  // Increased intensity for better illumination
                range: 500.0,       // Range that works well for large models
                color: Color::WHITE,
                shadows_enabled: true,
                ..default()
            },
            // Position the light slightly offset in front and above the camera direction
            transform: Transform::from_translation(Vec3::new(0.0, 50.0, -100.0)),
            ..default()
        });
    });

    setup_ui(commands);
}

fn setup_ui(mut commands: Commands) {
    debug!("Setting up UI");

    let style = TextStyle {
        font_size: 20.0,
        color: Color::WHITE,
        ..default()
    };

    let stats_style = TextStyle {
        font_size: 16.0,
        color: Color::WHITE,
        ..default()
    };

    // Main UI container
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            // Top middle operation display
            parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::FlexStart,
                            padding: UiRect::top(Val::Px(20.)),
                            ..default()
                        },
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle::from_section("Current Operation: None", style.clone()),
                        OperationText, // Marker component to update this text later
                    ));
                });

            // Top left statistics display
            parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            top: Val::Px(20.),
                            left: Val::Px(20.),
                            ..default()
                        },
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle::from_section("Vertices: 0 | Edges: 0", stats_style.clone()),
                        StatsText, // Marker component to update this text later
                    ));
                });

            // Bottom left instructions
            parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            bottom: Val::Px(20.),
                            left: Val::Px(20.),
                            ..default()
                        },
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn(
                        TextBundle::from_sections(vec![
                            TextSection::new("Space - Cycle boolean op\n", style.clone()),
                            TextSection::new("C - Cycle secondary shape\n", style.clone()),
                            TextSection::new("Q - Quit with error message\n", style.clone()),
                        ])
                        .with_style(Style { ..default() }),
                    );
                });
        });
}

// Helper function to calculate the bounding box min/max of a mesh
fn calculate_mesh_min_max(mesh: &Mesh) -> (Vec3, Vec3) {
    use bevy::render::mesh::VertexAttributeValues;
    
    if let Some(positions) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        match positions {
            VertexAttributeValues::Float32x3(positions) => {
                if positions.is_empty() {
                    return (Vec3::ZERO, Vec3::ZERO);
                }
                
                let mut min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
                let mut max = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
                
                for pos in positions.iter() {
                    let p = Vec3::new(pos[0], pos[1], pos[2]);
                    min = min.min(p);
                    max = max.max(p);
                }
                
                (min, max)
            }
            _ => {
                // If positions are not in the expected format, return a default
                (Vec3::ZERO, Vec3::ONE)
            }
        }
    } else {
        // If no positions attribute found, return a default
        (Vec3::ZERO, Vec3::ONE)
    }
}

#[derive(Component)]
struct StepModel;

fn load_step_and_setup_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut has_loaded: Local<bool>,
    cli_args: Res<CliArgs>,
) {
    if *has_loaded {
        return;
    }
    
    log::debug!("Starting STEP model loading and mesh setup");
    
    // Determine which STEP file to load
    let step_file_path = cli_args.step_file.as_deref().unwrap_or("assets/real_parts/multifeature.step".as_ref());

    // Load the STEP file directly using the path loading function
    let step_asset = match bevy_step_loader::load_step_file_from_path(step_file_path) {
        Ok(asset) => {
            log::debug!("Successfully loaded STEP file from path: {:?}", step_file_path);
            asset
        },
        Err(e) => {
            log::error!("Failed to load STEP file from path: {}: {}", step_file_path.display(), e);
            // Fallback to cube
            StepAsset {
                mesh: Cuboid::new(1.2, 1.2, 1.2).mesh().build(), // Fixed type mismatch
            }
        }
    };

    // Calculate the STEP model's bounds to determine proper camera positioning and scaling
    let (step_min, step_max) = calculate_mesh_min_max(&step_asset.mesh);
    let step_model_size = step_max - step_min;
    let max_dimension = step_model_size.max_element();
    
    // Calculate camera distance to have the STEP model occupy ~80% of the FOV
    // For a perspective camera with default FOV, we use the diagonal of the model's bounding box
    let model_diagonal = step_model_size.length();
    let desired_fov_ratio = 0.8; // 80% FOV occupancy
    // For a 60-degree FOV (default), distance = (model_size / 2) / tan(FOV/2)
    // Since the diagonal is the largest dimension, we'll use a factor to account for it
    let camera_distance = (model_diagonal * 0.5) / (std::f32::consts::FRAC_PI_3 / 2.0).tan() / desired_fov_ratio;
    
    // Sanity check: if the model is extremely small, use a reasonable default
    let camera_distance = if camera_distance < 10.0 { 50.0 } else { camera_distance };
    
    // Position the camera to look at the center of the STEP model
    let model_center = (step_min + step_max) * 0.5;
    
    // Update the orbit state with the proper center and distance
    commands.insert_resource(OrbitState {
        center: model_center,
        distance: camera_distance.max(10.0), // Ensure minimum distance
        angle: 0.0,
    });
    
    // Create materials with improved metallic/aluminum-like properties
    let step_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.8, 0.8), // Light gray
        metallic: 0.95, // High metallic value for aluminum-like appearance
        perceptual_roughness: 0.1, // Low roughness for shiny surface
        reflectance: 0.9, // High reflectance
        ..default()
    });
    
    let secondary_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.5, 0.9), // Blue
        metallic: 0.5, // Moderate metallic value
        perceptual_roughness: 0.4, // Moderate roughness
        ..default()
    });
    
    let result_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.8, 0.4), // Green
        metallic: 0.3, // Lower metallic value to distinguish from original
        perceptual_roughness: 0.5, // Slightly more rough
        ..default()
    });

    // Calculate the offset needed to center the STEP model at the origin
    let step_model_center = (step_min + step_max) * 0.5;
    let centering_offset = -step_model_center;
    
    // Spawn the primary STEP model centered at origin for predictable positioning
    let step_mesh_handle = meshes.add(step_asset.mesh.clone());
    let step_entity = commands.spawn((
        PbrBundle {
            mesh: step_mesh_handle.clone(),
            material: step_material,
            transform: Transform::from_translation(centering_offset), // Center the mesh at origin
            ..default()
        },
        StepModel,
        PrimaryBooleanMesh {
            secondary_entity: Entity::PLACEHOLDER, // Will be set after spawning secondary
        },
    )).id();
    
    // Update the center calculation to be at origin now
    let step_center = Vec3::ZERO; // Since we're centering the model at origin
    log::debug!("STEP model centered at origin. Centering offset applied: {:?}", centering_offset);

    // Calculate the secondary shape size - should be about 60% of the largest dimension of the STEP model for good intersection
    // This ensures it's large enough to intersect properly with the STEP model
    let secondary_base_size = max_dimension * 0.6;  // 60% of the largest dimension (increased from 40%)
    
    log::debug!("Calculated secondary size: {}", secondary_base_size);
    
    // Position the secondary shape to intersect well with the STEP model
    // Since the STEP model is now centered at origin, position the secondary shape at origin with a slight offset
    let secondary_pos = step_center + Vec3::new(max_dimension * 0.1, max_dimension * 0.1, max_dimension * 0.1); // Slight offset from center
    
    log::debug!("Positioning - STEP center: {:?}, Secondary position: {:?}", step_center, secondary_pos);
    
    // Log debug information about sizing for verification
    log::debug!("STEP model sizing - Max dimension: {}, Secondary size: {}", max_dimension, secondary_base_size);
    log::debug!("Positioning - STEP center: {:?}, Secondary position: {:?}", step_center, secondary_pos);
    
    // Sanity check: Add a panic if the secondary shape is too small to be meaningful
    if secondary_base_size < 0.1 {
        panic!("Fatal error: Secondary shape is extremely small ({}). This indicates a problem with STEP model bounds calculation.", secondary_base_size);
    }
    
    // Additional debug: Show the actual STEP model bounds
    log::debug!("STEP model bounds - Min: {:?}, Max: {:?}, Size: {:?}", step_min, step_max, step_model_size);
    
    // Sanity check: Ensure the secondary position makes sense relative to the STEP model
    let position_offset = (secondary_pos - step_center).length();
    if position_offset > max_dimension * 2.0 {
        log::warn!("Warning: Secondary shape is positioned very far ({}) from STEP model center. This may result in poor intersections.", position_offset);
    }
    
    // Create the initial secondary shape (cube) at the correct size
    // Instead of creating a unit cube and scaling it, create it at the proper size directly
    // This avoids issues with the boolean operation system not applying scaling correctly
    let secondary_mesh = meshes.add(Cuboid::new(secondary_base_size, secondary_base_size, secondary_base_size));
    let secondary_transform = Transform::from_translation(secondary_pos);
    log::debug!("Creating secondary shape with correct size:");
    log::debug!("  Dimensions: {} x {} x {}", secondary_base_size, secondary_base_size, secondary_base_size);
    log::debug!("  Position: {:?}", secondary_pos);
    log::debug!("  Full transform: {:?}", secondary_transform);
    log::debug!("  STEP model center: {:?}", step_center);
    log::debug!("  Distance from STEP center: {}", (secondary_pos - step_center).length());
    
    let secondary_entity = commands.spawn((
        PbrBundle {
            mesh: secondary_mesh.clone(),
            material: secondary_material,
            transform: secondary_transform,
            ..default()
        },
        SecondaryBooleanMesh {
            primary_entity: step_entity, // Will be set after spawning secondary
        },
    )).id();

    // Update the references to each other
    commands.entity(step_entity).insert(PrimaryBooleanMesh {
        secondary_entity,
    });

    commands.entity(secondary_entity).insert(SecondaryBooleanMesh {
        primary_entity: step_entity,
    });

    // Create result entity (hidden initially) - positioned at the center where boolean result appears
    let result_entity = commands.spawn((
        PbrBundle {
            material: result_material,
            visibility: Visibility::Hidden, // Initially hidden
            transform: Transform::from_xyz(0.0, 0.0, 0.0), // Center position for result
            ..default()
        },
        ResultShape,
    )).id();

    // Set up the boolean operation handles
    commands.insert_resource(BooleanHandles {
        primary_entity: step_entity,
        secondary_entity,
        result_entity,
    });

    // Store the original scaling information for shape cycling
    commands.insert_resource(SecondaryShapeScale {
        original_scale: secondary_base_size,
    });

    // Store handles for later use
    commands.insert_resource(DemoHandles {
        step_shape: step_entity,
        secondary_shape: secondary_entity,
        result: result_entity,
        original_step_mesh: step_mesh_handle,
        original_secondary_mesh: secondary_mesh.clone(),
    });

    // Initialize the boolean operation state from CLI args later in the setup
    // This will be handled by a dedicated system after resources are set up

    log::debug!("Spawned entities - STEP model: {:?}, Secondary: {:?}, Result: {:?}", 
               step_entity, secondary_entity, result_entity);

    *has_loaded = true;
    log::debug!("Completed setup with STEP model and secondary shape");
}

// This system cycles through the boolean operations when the spacebar is pressed
fn cycle_boolean_op(
    keys: Res<ButtonInput<KeyCode>>,
    mut op_state: ResMut<BooleanOpState>,
) {
    if keys.just_pressed(KeyCode::Space) {
        *op_state = match *op_state {
            BooleanOpState::None => BooleanOpState::Intersect,
            BooleanOpState::Intersect => BooleanOpState::Union,
            BooleanOpState::Union => BooleanOpState::Subtract,
            BooleanOpState::Subtract => BooleanOpState::None,
        };
        debug!("BooleanOpState changed to: {:?}", *op_state);
    }
}

// This system cycles through the secondary shapes when 'C' key is pressed
fn cycle_secondary_shape(
    keys: Res<ButtonInput<KeyCode>>,
    mut shape_state: ResMut<SecondaryShape>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut secondary_query: Query<(&mut Handle<Mesh>, &mut Transform), With<SecondaryBooleanMesh>>,
    mut boolean_op_state: ResMut<BooleanOpState>,
    shape_scale: Res<SecondaryShapeScale>,
) {
    if keys.just_pressed(KeyCode::KeyC) {
        *shape_state = match *shape_state {
            SecondaryShape::Cube => SecondaryShape::Sphere,
            SecondaryShape::Sphere => SecondaryShape::Cube,
        };
        debug!("SecondaryShape changed to: {:?}", *shape_state);
        
        // Update the secondary entity's mesh
        if let Ok((mut mesh_handle, mut transform)) = secondary_query.get_single_mut() {
            // Preserve the entire current transform including position and rotation
            let current_transform = *transform;
            // Use the original scale from the resource, not the current transform scale
            let proper_scale = shape_scale.original_scale;
            
            match *shape_state {
                SecondaryShape::Cube => {
                    // Create a cube mesh at the correct size directly
                    // Use the original scale as the size reference
                    let cube_size = proper_scale;
                    let cube_mesh = meshes.add(Cuboid::new(cube_size, cube_size, cube_size)); // Cube with proper size
                    *mesh_handle = cube_mesh;
                }
                SecondaryShape::Sphere => {
                    // Create a sphere mesh - make it the same "size" as the cube by using diameter of cube_size
                    let sphere_radius = proper_scale * 0.5; // Radius = half of cube side length
                    let sphere_mesh = meshes.add(Sphere::new(sphere_radius)); // Radius to match unit cube
                    *mesh_handle = sphere_mesh;
                }
            }
            
            // Restore the entire transform to preserve positioning
            // But reset the scale to (1,1,1) since we're now creating meshes at the correct size
            *transform = Transform {
                translation: current_transform.translation, // Keep the same position
                rotation: current_transform.rotation,       // Keep the same rotation
                scale: Vec3::ONE,                          // Reset scale since size is now in the mesh
            };
            
            // Log the new positioning for debugging
            debug!("Cycled secondary shape - Position: {:?}, Original scale: {}", 
                   current_transform.translation, proper_scale);
            
            // Trigger a recomputation by marking the boolean operation state as changed
            // This forces the boolean operation system to recompute with the new geometry
            boolean_op_state.set_changed();
        }
    }
}

// This system updates the UI text to show the current operation
fn update_operation_text(
    op_state: Res<BooleanOpState>,
    mut query: Query<&mut Text, With<OperationText>>,
) {
    if op_state.is_changed() {
        let text = match *op_state {
            BooleanOpState::None => "Current Operation: None",
            BooleanOpState::Intersect => "Current Operation: Intersect",
            BooleanOpState::Union => "Current Operation: Union",
            BooleanOpState::Subtract => "Current Operation: Subtract",
        };

        for mut text_component in query.iter_mut() {
            text_component.sections[0].value = text.to_string();
        }
    }
}

// This system updates the UI text to show current geometry statistics
fn update_stats_text(
    stats: Res<GeometryStats>,
    mut query: Query<&mut Text, With<StatsText>>,
) {
    if stats.is_changed() {
        log::debug!("Updating stats text with: vertices={}, edges={}", stats.vertices, stats.edges);
        if let Ok(mut text) = query.get_single_mut() {
            text.sections[0].value =
                format!("Vertices: {} | Edges: {}", stats.vertices, stats.edges);
        }
    }
}

// System for orbit camera
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

// System to update orbit state (slowly rotate camera)
fn update_orbit_state(
    mut orbit_state: ResMut<OrbitState>,
) {
    orbit_state.angle += 0.005; // Slowly rotate the camera
}

// This system exits the app when 'q' is pressed with error message
fn exit_on_q_key(
    keys: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::KeyQ) {
        eprintln!("User did not see expected results");
        exit.send(AppExit::Success);
    }
}

// System to set the initial boolean operation state from CLI args
fn set_initial_op_state(
    mut op_state: ResMut<BooleanOpState>,
    cli_args: Res<CliArgs>,
    mut has_set_initial_state: Local<bool>,
) {
    if !*has_set_initial_state {
        *op_state = cli_args.initial_boolean_op;
        log::debug!("Set initial boolean operation state to: {:?}", *op_state);
        *has_set_initial_state = true;
    }
}