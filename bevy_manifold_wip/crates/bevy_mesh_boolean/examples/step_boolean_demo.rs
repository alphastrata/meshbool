use bevy::{
    core_pipeline::tonemapping::Tonemapping,
    prelude::*,
};
use clap::Parser;
use std::path::PathBuf;
use log;

use bevy_step_loader;

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

// A marker component for the operation text UI element
#[derive(Component)]
struct OperationText;

// A marker component for the statistics text UI element
#[derive(Component)]
struct StatsText;

// A resource to hold the current state of the boolean operation
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
enum BooleanOpState {
    #[default]
    None,
    Intersect,
    Union,
    Subtract,
}

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

// CLI arguments structure
#[derive(Parser)]
#[clap(name = "STEP Boolean Demo", about = "A demo combining STEP file loading with boolean operations")]
struct Args {
    /// Path to the STEP file
    #[clap(long, value_parser)]
    step_file: Option<PathBuf>,
    
    /// Boolean operation to perform
    #[clap(long, value_parser, default_value = "none")]
    boolean_op: String,
}

// A resource to hold current geometry statistics
#[derive(Resource, Default, Debug, Clone, Copy)]
struct GeometryStats {
    vertices: usize,
    edges: usize,
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

fn main() {
    // Initialize logger
    env_logger::init();
    
    // Parse command line arguments
    let args = Args::parse();
    
    log::debug!("Parsed CLI args: step_file: {:?}, boolean_op: {}", 
           args.step_file, args.boolean_op);
    
    // Initialize the boolean operation state based on CLI argument
    let initial_op = match args.boolean_op.as_str() {
        "intersection" => BooleanOpState::Intersect,
        "subtract" => BooleanOpState::Subtract,
        "union" => BooleanOpState::Union,
        _ => BooleanOpState::None,
    };
    
    log::debug!("Initial boolean operation state: {:?}", initial_op);

    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            bevy_mesh_boolean::MeshBooleanPlugin,
            bevy_step_loader::StepPlugin,
        ))
        .insert_resource(ClearColor(Color::srgb(0.15, 0.15, 0.15)))
        .insert_resource(AmbientLight {
            brightness: 0.75,
            color: Color::WHITE,
        })
        // Add the new resources
        .insert_resource(CliArgs {
            step_file: args.step_file,
            initial_boolean_op: initial_op,
        })
        .insert_resource(initial_op)
        .insert_resource(SecondaryShape::default())
        .insert_resource(OrbitState {
            angle: 0.0,
            center: Vec3::ZERO,
            distance: 10.0,
        })
        .init_resource::<GeometryStats>()
        .add_systems(Startup, (setup, setup_demo).chain())
        .add_systems(
            Update,
            (
                cycle_boolean_op,
                cycle_secondary_shape,
                update_operation_text,
                update_stats_text,
                apply_boolean_operations,
                orbit_camera,
                update_orbit_state,
            ).chain(),
        )
        .run();
}

fn setup(mut commands: Commands) {
    debug!("Setting up the scene");

    // Add lighting
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 3000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::YXZ,
            150.0_f32.to_radians(),
            -40.0_f32.to_radians(),
            0.0,
        )),
        ..default()
    });

    // Position camera to see the spawned shapes
    let cam_trans =
        Transform::from_xyz(0.0, 3.0, 10.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y);

    commands.spawn((
        Camera3dBundle {
            transform: cam_trans,
            tonemapping: Tonemapping::AcesFitted,
            ..default()
        },
        OrbitCamera,
    ));

    setup_ui(commands);
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

fn setup_demo(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    secondary_shape_state: Res<SecondaryShape>,
    cli_args: Res<CliArgs>,
    op_state: Res<BooleanOpState>,
) {
    log::debug!("Setting up demo with secondary shape: {:?} and CLI args: {:?}", 
                *secondary_shape_state, *cli_args);

    // Create secondary shape mesh based on the current shape state
    let secondary_mesh = match *secondary_shape_state {
        SecondaryShape::Cube => meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        SecondaryShape::Sphere => meshes.add(Sphere::new(0.8)),
    };

    // Load the STEP file if provided, otherwise use a cube as fallback
    let step_handle = if let Some(step_file_path) = &cli_args.step_file {
        log::debug!("Loading STEP file: {:?}", step_file_path);
        let path_str = step_file_path.to_str().expect("Invalid STEP file path");
        asset_server.load(path_str.to_string())
    } else {
        log::debug!("No STEP file provided, using cube as fallback");
        // For now, just create a cube as a fallback
        meshes.add(Cuboid::new(1.2, 1.2, 1.2))
    };

    // Create materials
    let step_material = materials.add(Color::srgb(0.9, 0.7, 0.3)); // Orange
    let secondary_material = materials.add(Color::srgb(0.3, 0.5, 0.9)); // Blue
    let result_material = materials.add(Color::srgb(0.6, 0.8, 0.4)); // Green

    let step_shape = commands
        .spawn(PbrBundle {
            mesh: step_handle.clone(),
            material: step_material,
            transform: Transform::from_xyz(-1.0, 0.0, 0.0),
            ..default()
        })
        .insert(bevy_mesh_boolean::PrimaryBooleanMesh {
            secondary_entity: Entity::PLACEHOLDER, // Will be set after spawning secondary
        })
        .insert(StepModel)
        .id();

    let secondary_shape = commands
        .spawn(PbrBundle {
            mesh: secondary_mesh.clone(),
            material: secondary_material,
            transform: Transform::from_xyz(1.0, 0.0, 0.0),
            ..default()
        })
        .insert(bevy_mesh_boolean::SecondaryBooleanMesh {
            primary_entity: step_shape, // Will be set after spawning secondary
        })
        .id();

    // Update the references to each other
    commands.entity(step_shape).insert(bevy_mesh_boolean::PrimaryBooleanMesh {
        secondary_entity: secondary_shape,
    });

    commands.entity(secondary_shape).insert(bevy_mesh_boolean::SecondaryBooleanMesh {
        primary_entity: step_shape,
    });

    let result = commands
        .spawn(PbrBundle {
            material: result_material,
            // The result mesh is initially hidden when no operation is applied
            visibility: if matches!(*op_state, BooleanOpState::None) {
                Visibility::Hidden
            } else {
                Visibility::Visible
            },
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        })
        .id();

    commands.insert_resource(DemoHandles {
        step_shape,
        secondary_shape,
        result,
        original_step_mesh: step_handle,
        original_secondary_mesh: secondary_mesh.clone(),
    });

    log::debug!("Demo setup complete with step_shape: {:?}, secondary_shape: {:?}", 
           step_shape, secondary_shape);
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
        .spawn((NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                position_type: PositionType::Absolute,
                ..default()
            },
            ..default()
        },))
        .with_children(|parent| {
            // Top middle operation display
            parent
                .spawn((NodeBundle {
                    style: Style {
                        width: Val::Percent(100.),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::FlexStart,
                        padding: UiRect::top(Val::Px(20.)),
                        ..default()
                    },
                    ..default()
                },))
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle::from_section("Current Operation: None", style.clone()),
                        OperationText, // Marker component to update this text later
                    ));
                });

            // Top left statistics display
            parent
                .spawn((NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        top: Val::Px(20.),
                        left: Val::Px(20.),
                        ..default()
                    },
                    ..default()
                },))
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle::from_section("Vertices: 0 | Edges: 0", stats_style.clone()),
                        StatsText, // Marker component to update this text later
                    ));
                });

            // Bottom left instructions
            parent
                .spawn((NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(20.),
                        left: Val::Px(20.),
                        ..default()
                    },
                    ..default()
                },))
                .with_children(|parent| {
                    parent.spawn(
                        TextBundle::from_sections(vec![
                            TextSection::new("Space - Cycle boolean op\\n", style.clone()),
                            TextSection::new("C - Cycle secondary shape\\n", style.clone()),
                        ])
                        .with_style(Style { ..default() }),
                    );
                });
        });
}

// This system cycles through the boolean operations when the spacebar is pressed
fn cycle_boolean_op(keys: Res<ButtonInput<KeyCode>>, mut op_state: ResMut<BooleanOpState>) {
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
fn cycle_secondary_shape(keys: Res<ButtonInput<KeyCode>>, mut shape_state: ResMut<SecondaryShape>) {
    if keys.just_pressed(KeyCode::KeyC) {
        *shape_state = match *shape_state {
            SecondaryShape::Cube => SecondaryShape::Sphere,
            SecondaryShape::Sphere => SecondaryShape::Cube,
        };
        debug!("SecondaryShape changed to: {:?}", *shape_state);
    }
}

// This system updates the UI text to show the current operation
fn update_operation_text(
    op_state: Res<BooleanOpState>,
    shape_state: Res<SecondaryShape>,
    mut query: Query<&mut Text, With<OperationText>>,
) {
    debug!("Updating operation text with state: {:?}, shape: {:?}", *op_state, *shape_state);

    if op_state.is_changed() {
        if let Ok(mut text) = query.get_single_mut() {
            let operation_name = match *op_state {
                BooleanOpState::None => "None",
                BooleanOpState::Intersect => "Intersection",
                BooleanOpState::Union => "Union",
                BooleanOpState::Subtract => "Subtraction",
            };

            let shape_name = match *shape_state {
                SecondaryShape::Cube => "Cube",
                SecondaryShape::Sphere => "Sphere",
            };

            text.sections[0].value = format!(
                "Operation: {} | Secondary Shape: {}",
                operation_name, shape_name
            );
            debug!("Updated operation text to: {}", text.sections[0].value);
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

// System to apply boolean operations and update stats
fn apply_boolean_operations(
    op_state: Res<BooleanOpState>,
    handles: Res<DemoHandles>,
    meshes: ResMut<Assets<Mesh>>,
    mut visibility_query: Query<&mut Visibility>,
    mut stats: ResMut<GeometryStats>,
    _commands: Commands,
    query: Query<(&Handle<Mesh>, &GlobalTransform), With<bevy_mesh_boolean::PrimaryBooleanMesh>>,
) {
    if op_state.is_changed() {
        log::debug!("BooleanOpState changed to: {:?}", *op_state);

        match *op_state {
            BooleanOpState::None => {
                // Show original shapes and hide result
                if let Ok(mut vis) = visibility_query.get_mut(handles.step_shape) {
                    *vis = Visibility::Visible;
                }
                if let Ok(mut vis) = visibility_query.get_mut(handles.secondary_shape) {
                    *vis = Visibility::Visible;
                }
                if let Ok(mut vis) = visibility_query.get_mut(handles.result) {
                    *vis = Visibility::Hidden;
                }
                
                log::debug!("Showing original shapes, hiding result");
            }
            _ => {
                // Hide original shapes and show result
                if let Ok(mut vis) = visibility_query.get_mut(handles.step_shape) {
                    *vis = Visibility::Hidden;
                }
                if let Ok(mut vis) = visibility_query.get_mut(handles.secondary_shape) {
                    *vis = Visibility::Hidden;
                }
                if let Ok(mut vis) = visibility_query.get_mut(handles.result) {
                    *vis = Visibility::Visible;
                }
                
                log::debug!("Hiding original shapes, showing result");
            }
        }
    }
    
    // Update stats based on current operation
    if let Ok((mesh_handle, _transform)) = query.get(handles.step_shape) {
        if let Some(mesh) = meshes.get(mesh_handle) {
            let vertex_count = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
                Some(bevy::render::mesh::VertexAttributeValues::Float32x3(positions)) => positions.len(),
                _ => 0,
            };
            
            if stats.vertices != vertex_count {
                stats.vertices = vertex_count;
                log::debug!("Updated vertex count to: {}", vertex_count);
            }
        }
    }
}
fn update_orbit_state(
    step_assets: Option<Res<Assets<bevy_step_loader::StepAsset>>>,
    handles: Res<DemoHandles>,
    step_query: Query<(&Handle<Mesh>, &GlobalTransform), With<StepModel>>,
    meshes: Res<Assets<Mesh>>,
    mut orbit_state: ResMut<OrbitState>,
    mut has_updated: Local<bool>,
) {
    if *has_updated {
        return; // Only update once
    }
    
    // Get the mesh handle for the step model entity
    if let Ok((step_mesh_handle, _transform)) = step_query.get(handles.step_shape) {
        if let Some(step_assets) = step_assets {
            // Check if this is a STEP asset by looking through all loaded STEP assets and seeing if the mesh matches
            for (_, step_asset) in step_assets.iter() {
                // Compare the mesh with the STEP asset mesh to see if they're the same
                if let Some(step_asset_positions) = step_asset.mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
                    if let Some(target_positions) = meshes.get(step_mesh_handle).and_then(|mesh| mesh.attribute(Mesh::ATTRIBUTE_POSITION)) {
                        // For now, we'll just check if there are positions and they're the same length as a simple heuristic
                        if let (bevy::render::mesh::VertexAttributeValues::Float32x3(step_asset_pos),
                                bevy::render::mesh::VertexAttributeValues::Float32x3(target_pos)) = 
                               (step_asset_positions, target_positions) {
                            if step_asset_pos.len() == target_pos.len() {
                                // Calculate the bounding box of the STEP model to center the orbit
                                if !step_asset_pos.is_empty() {
                                    let mut min_bound = Vec3::new(step_asset_pos[0][0], step_asset_pos[0][1], step_asset_pos[0][2]);
                                    let mut max_bound = min_bound;
                                    
                                    for vertex in step_asset_pos.iter() {
                                        let v = Vec3::new(vertex[0], vertex[1], vertex[2]);
                                        min_bound = min_bound.min(v);
                                        max_bound = max_bound.max(v);
                                    }
                                    
                                    orbit_state.center = (min_bound + max_bound) * 0.5;
                                    let model_size = (max_bound - min_bound).length();
                                    orbit_state.distance = model_size.max(5.0) * 1.5; // Adjust camera distance based on model size
                                    *has_updated = true;
                                    
                                    log::debug!("Updated orbit state - center: {:?}, distance: {}", orbit_state.center, orbit_state.distance);
                                    return;
                                }
                            }
                        }
                    }
                }
            }
        }
        // If we can't get STEP assets, we can still try to calculate the bounds from the regular mesh
        else if let Some(mesh) = meshes.get(step_mesh_handle) {
            if let Some(positions) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
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
                        *has_updated = true;
                        
                        log::debug!("Updated orbit state from regular mesh - center: {:?}, distance: {}", orbit_state.center, orbit_state.distance);
                        return;
                    }
                }
            }
        }
    }
}
