// In examples/step_to_step_subtract.rs
use bevy::prelude::*;

use bevy_mesh_boolean::*;
use bevy_step_loader::*;
use clap::Parser;

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

#[derive(Parser, Resource)]
#[command(name = "step-to-step-subtract", about = "Subtract one STEP file from another")]
struct Args {
    /// Base STEP file to subtract from
    #[arg(long, default_value = "real_parts/multifeature.step")]
    base: String,
    
    /// STEP file to subtract with (cut tool)
    #[arg(long, default_value = "real_parts/cube.step")]
    cut_with: String,
}

fn main() {
    let args = Args::parse();
    
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            MeshBooleanPlugin,
            StepPlugin,
        ))
        .insert_resource(args)
        .add_systems(Startup, setup)
        .add_systems(Update, (load_step_files_and_perform_subtract, update_ui_text, toggle_boolean_op, orbit_camera))
        .run();
}

#[derive(Resource)]
struct StepFilesResource {
    base_file: String,
    cut_with_file: String,
}

#[derive(Resource, Default)]
struct BooleanOperationState {
    base_loaded: bool,
    cut_with_loaded: bool,
    entities_created: bool,
    boolean_performed: bool,
}

#[derive(Component)]
struct InfoText;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    args: Res<Args>,
) {
    eprintln!("[SETUP] Loading STEP files: base={}, cut_with={}", args.base, args.cut_with);
    
    // Load the STEP files
    let base_handle: Handle<StepAsset> = asset_server.load(&args.base);
    let cut_with_handle: Handle<StepAsset> = asset_server.load(&args.cut_with);
    
    commands.insert_resource(StepFilesResource {
        base_file: args.base.clone(),
        cut_with_file: args.cut_with.clone(),
    });
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
            intensity: 1500.0,
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
            "Loading STEP files...",
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
    
    eprintln!("[SETUP] Completed STEP file loading setup");
}

fn orbit_camera(
    mut query: Query<&mut Transform, With<OrbitCamera>>,
    mut orbit_state: ResMut<OrbitState>,
) {
    orbit_state.angle += 0.01; // Slowly rotate the camera
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

fn load_step_files_and_perform_subtract(
    mut commands: Commands,
    step_assets: Res<Assets<StepAsset>>,
    asset_server: Res<AssetServer>,
    step_files_resource: Res<StepFilesResource>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut boolean_state: ResMut<BooleanOperationState>,
    mut text_query: Query<&mut Text, With<InfoText>>,
    mut orbit_state: ResMut<OrbitState>,
) {
    if boolean_state.boolean_performed {
        return;
    }
    
    // Load base STEP file
    let base_handle: Handle<StepAsset> = asset_server.load(&step_files_resource.base_file);
    let base_loaded = step_assets.get(&base_handle).is_some();
    eprintln!("[TRACE] Base STEP file loaded status: {}", base_loaded);
    
    // Load cut_with STEP file
    let cut_with_handle: Handle<StepAsset> = asset_server.load(&step_files_resource.cut_with_file);
    let cut_with_loaded = step_assets.get(&cut_with_handle).is_some();
    eprintln!("[TRACE] Cut_with STEP file loaded status: {}", cut_with_loaded);
    
    if base_loaded && !boolean_state.base_loaded {
        // Update UI
        if let Ok(mut text) = text_query.get_single_mut() {
            text.sections[0].value = "Base STEP file loaded! Loading cut_with file...".to_string();
        }
        
        eprintln!("[STEP-TO-STEP] Base STEP file loaded successfully!");
        boolean_state.base_loaded = true;
    }
    
    if cut_with_loaded && !boolean_state.cut_with_loaded {
        // Update UI
        if let Ok(mut text) = text_query.get_single_mut() {
            text.sections[0].value = "Cut_with STEP file loaded! Setting up boolean operation...".to_string();
        }
        
        eprintln!("[STEP-TO-STEP] Cut_with STEP file loaded successfully!");
        boolean_state.cut_with_loaded = true;
    }
    
    if base_loaded && cut_with_loaded && !boolean_state.entities_created {
        let base_asset = step_assets.get(&base_handle).unwrap();
        let cut_with_asset = step_assets.get(&cut_with_handle).unwrap();
        
        // Get mesh statistics
        let base_vertex_count = if let Some(positions) = base_asset.mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            match positions {
                bevy::render::mesh::VertexAttributeValues::Float32x3(pos) => pos.len(),
                _ => 0,
            }
        } else {
            0
        };
        
        let cut_with_vertex_count = if let Some(positions) = cut_with_asset.mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            match positions {
                bevy::render::mesh::VertexAttributeValues::Float32x3(pos) => pos.len(),
                _ => 0,
            }
        } else {
            0
        };
        
        eprintln!("[STEP-TO-STEP] Base mesh has {} vertices, Cut_with mesh has {} vertices", 
                 base_vertex_count, cut_with_vertex_count);
        
        // Calculate model center for camera orbit
        // Use the base model for camera centering
        if let Some(base_positions) = base_asset.mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            if let bevy::render::mesh::VertexAttributeValues::Float32x3(pos) = base_positions {
                if !pos.is_empty() {
                    let mut min_bound = Vec3::new(pos[0][0], pos[0][1], pos[0][2]);
                    let mut max_bound = min_bound;
                    
                    for vertex in pos.iter() {
                        let v = Vec3::new(vertex[0], vertex[1], vertex[2]);
                        min_bound = min_bound.min(v);
                        max_bound = max_bound.max(v);
                    }
                    
                    orbit_state.center = (min_bound + max_bound) * 0.5;
                    let base_model_size = (max_bound - min_bound).length();
                    let cut_with_model_size = if let Some(cut_positions) = cut_with_asset.mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
                        if let bevy::render::mesh::VertexAttributeValues::Float32x3(cut_pos) = cut_positions {
                            if !cut_pos.is_empty() {
                                let mut min_cut = Vec3::new(cut_pos[0][0], cut_pos[0][1], cut_pos[0][2]);
                                let mut max_cut = min_cut;
                                
                                for vertex in cut_pos.iter() {
                                    let v = Vec3::new(vertex[0], vertex[1], vertex[2]);
                                    min_cut = min_cut.min(v);
                                    max_cut = max_cut.max(v);
                                }
                                
                                (max_cut - min_cut).length()
                            } else {
                                base_model_size
                            }
                        } else {
                            base_model_size
                        }
                    } else {
                        base_model_size
                    };
                    
                    let max_model_size = base_model_size.max(cut_with_model_size);
                    orbit_state.distance = max_model_size.max(5.0) * 1.5; // Adjust camera distance based on model size
                }
            }
        }
        
        // Update UI with stats
        if let Ok(mut text) = text_query.get_single_mut() {
            text.sections[0].value = format!("Base: {} vertices, Cut_with: {} vertices. Performing subtraction...", 
                                            base_vertex_count, cut_with_vertex_count);
        }
        
        // Spawn the base STEP model
        let base_mesh_handle = meshes.add(base_asset.mesh.clone());
        let base_material = materials.add(Color::srgb(0.8, 0.7, 0.6));
        
        let base_entity = commands.spawn((
            PbrBundle {
                mesh: base_mesh_handle,
                material: base_material,
                transform: Transform::from_xyz(-0.5, 0.0, 0.0),
                ..default()
            },
            StepModel,
        )).id();
        
        // Spawn the cut_with STEP model
        let cut_with_mesh_handle = meshes.add(cut_with_asset.mesh.clone());
        let cut_with_material = materials.add(Color::srgb(0.6, 0.8, 0.9));
        
        let cut_with_entity = commands.spawn((
            PbrBundle {
                mesh: cut_with_mesh_handle,
                material: cut_with_material,
                transform: Transform::from_xyz(0.5, 0.0, 0.0),
                ..default()
            },
            StepModel,
        )).id();
        
        // Create result entity (hidden initially)
        let result_entity = commands.spawn(PbrBundle {
            material: materials.add(Color::srgb(0.9, 0.5, 0.5)), // Different color for result
            visibility: Visibility::Hidden, // Initially hidden
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        }).id();
        
        // Set up the boolean operation handles
        commands.insert_resource(BooleanHandles {
            primary_entity: base_entity,
            secondary_entity: cut_with_entity,
            result_entity,
        });
        
        // Mark the entities as created
        boolean_state.entities_created = true;
        eprintln!("[STEP-TO-STEP] Entities created - Base: {:?}, Cut_with: {:?}, Result: {:?}", 
                 base_entity, cut_with_entity, result_entity);
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
        
        eprintln!("[STEP-TO-STEP] Boolean subtract operation initiated!");
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