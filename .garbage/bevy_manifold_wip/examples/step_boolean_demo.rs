use bevy::prelude::*;
use bevy_mesh_boolean::*;
use bevy_step_loader::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            MeshBooleanPlugin, 
            StepPlugin,
        ))
        .add_systems(Startup, (setup, setup_step_data))
        .add_systems(Update, (setup_boolean_operation, cycle_boolean_op, update_operation_text))
        .run();
}

#[derive(Component)]
struct OperationText;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Load the STEP file
    let step_handle: Handle<StepAsset> = asset_server.load("real_parts/multifeature.step");
    
    // Create a cube to perform boolean operations with
    let cube_mesh = meshes.add(Cuboid::from_size(Vec3::splat(2.0))); // Make it large enough to intersect
    let cube_material = materials.add(Color::srgb(0.8, 0.7, 0.6));
    
    // Create a result material
    let result_material = materials.add(Color::srgb(0.9, 0.5, 0.5));
    
    // The STEP model will be spawned after it loads, so we'll set up our boolean operation system
    // to work with the loaded STEP model when it becomes available
    
    // For now, let's create a temporary setup that we'll replace when the STEP loads
    commands.insert_resource(StepData {
        step_handle,
        cube_mesh,
        cube_material,
        result_material,
    });

    // Camera and lighting setup
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 2.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // UI Text to show current operation
    commands.spawn((
        TextBundle::from_section(
            "Current Operation: None (Press Space to cycle: None -> Intersect -> Union -> Subtract -> None)",
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
        OperationText,
    ));
}

#[derive(Resource)]
struct StepData {
    step_handle: Handle<StepAsset>,
    cube_mesh: Handle<Mesh>,
    cube_material: Handle<StandardMaterial>,
    result_material: Handle<StandardMaterial>,
}

fn setup_boolean_operation(
    mut commands: Commands,
    step_assets: Res<Assets<StepAsset>>,
    step_data: Res<StepData>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut boolean_op_created: ResMut<ShouldCreateBooleanOp>,
) {
    if boolean_op_created.0 {
        return; // Already created
    }
    
    if let Some(step_asset) = step_assets.get(&step_data.step_handle) {
        eprintln!("STEP asset loaded! Creating boolean operation...");
        eprintln!("STEP mesh vertex count: {}", 
            if let Some(positions) = step_asset.mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
                match positions {
                    bevy::render::mesh::VertexAttributeValues::Float32x3(pos) => pos.len(),
                    _ => 0,
                }
            } else {
                0
            });
        
        // Create the boolean operation between the loaded STEP model and a cube
        let primary_mesh_handle = meshes.add(step_asset.mesh.clone());
        
        let bundle = MeshBooleanPlugin::spawn_boolean_operation(
            &mut commands,
            &mut meshes,
            &mut materials,
            primary_mesh_handle,           // STEP model mesh
            step_data.cube_mesh.clone(),   // Cube mesh
            Transform::from_xyz(0.0, 0.0, 0.0), // Position for STEP model
            Transform::from_xyz(1.0, 0.0, 0.0), // Position cube to intersect
            step_data.result_material.clone(),
        );
        
        eprintln!("Boolean operation created with entities:");
        eprintln!("  Primary (STEP): {:?}", bundle.primary);
        eprintln!("  Secondary (Cube): {:?}", bundle.secondary);
        eprintln!("  Result: {:?}", bundle.result);
        
        boolean_op_created.0 = true;
    }
}

#[derive(Resource, Deref, DerefMut, Default)]
struct ShouldCreateBooleanOp(bool);

fn cycle_boolean_op(
    mut op_state: ResMut<BooleanOpState>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        *op_state = match *op_state {
            BooleanOpState::None => BooleanOpState::Intersect,
            BooleanOpState::Intersect => BooleanOpState::Union,
            BooleanOpState::Union => BooleanOpState::Subtract,
            BooleanOpState::Subtract => BooleanOpState::None,
        };
    }
}

fn update_operation_text(
    op_state: Res<BooleanOpState>,
    mut query: Query<&mut Text, With<OperationText>>,
) {
    if !op_state.is_changed() {
        return;
    }

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

// Startup system to initialize our tracking resource
fn setup_step_data(
    mut commands: Commands,
) {
    commands.insert_resource(ShouldCreateBooleanOp(false));
}