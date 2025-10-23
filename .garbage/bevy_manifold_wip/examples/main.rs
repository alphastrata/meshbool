// In /examples/main.rs
use bevy::prelude::*;
use bevy_mesh_boolean::*;
use bevy_step_loader::*;
use clap::Parser;

#[derive(Parser)]
#[command(name = "bevy-boolean-step", about = "Load STEP files and perform boolean operations")]
struct Args {
    /// Path to the STEP file to load
    #[arg(default_value = "real_parts/LN_032.step")]
    step_file: String,
    
    /// Boolean operation to perform
    #[arg(long, default_value = "none", value_enum)]
    operation: BooleanOperation,
    
    /// Size of the cutting cube relative to the STEP model
    #[arg(long, default_value = "0.5")]
    cube_size_factor: f32,
    
    /// Position offset for the cutting cube
    #[arg(long, default_value = "2.0")]
    cube_offset: f32,
}

#[derive(clap::ValueEnum, Clone, Default, Debug)]
enum BooleanOperation {
    #[default]
    None,
    Subtract,
    Intersect,
    Union,
}

fn main() {
    let args = Args::parse();
    
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            MeshBooleanPlugin,
            StepPlugin,
        ))
        .insert_resource(ArgsResource(args))
        .insert_resource(BooleanOpState::None) // Initialize with default state 
        .add_systems(Startup, setup)
        .add_systems(Update, (perform_boolean_op_on_loaded_meshes, update_step_mesh))
        .run();
}

#[derive(Resource)]
struct ArgsResource(Args);

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    args_resource: Res<ArgsResource>,
) {
    let args = &args_resource.0;
    
    // Load the STEP file specified via command line
    let step_handle: Handle<bevy_step_loader::StepAsset> = asset_server.load(&args.step_file);
    
    // Material for the STEP model
    let step_material = materials.add(StandardMaterial::from(Color::srgb(0.9, 0.7, 0.3)));
    
    // Create a placeholder mesh for the STEP model initially
    let placeholder_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    
    // Spawn the STEP model as the primary mesh for boolean operations
    let step_entity = commands.spawn((
        PbrBundle {
            mesh: placeholder_mesh,
            material: step_material,
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        // Add the BooleanMesh component to mark this as the primary entity
        BooleanMesh { 
            op: MeshBooleanOp::Primary, 
            secondary_entity: Entity::PLACEHOLDER // Will be updated when cube entity is created
        }
    )).id();

    // We'll create a cube that's a factor of the estimated STEP model size
    let estimated_step_size = Vec3::splat(4.0); // Estimated size of the STEP model
    let cube_size = estimated_step_size * args.cube_size_factor; // Cube will be a factor of the STEP model size
    let cube_mesh = meshes.add(Cuboid::new(cube_size.x, cube_size.y, cube_size.z));
    
    // Material for the cutting cube
    let cube_material = materials.add(StandardMaterial::from(Color::srgb(0.3, 0.5, 0.9)));
    
    // Spawn the cutting cube
    let cube_entity = commands.spawn((
        PbrBundle {
            mesh: cube_mesh,
            material: cube_material,
            transform: Transform::from_xyz(args.cube_offset, 0.0, 0.0), // Offset to make the cut visible
            ..default()
        },
        // Add the BooleanMesh component to mark this as the secondary entity
        BooleanMesh { 
            op: MeshBooleanOp::Secondary, 
            secondary_entity: step_entity 
        }
    )).id();
    
    // Now update the primary entity's BooleanMesh to reference the cube entity
    commands.entity(step_entity).insert(BooleanMesh { 
        op: MeshBooleanOp::Primary, 
        secondary_entity: cube_entity 
    });

    // Set the boolean operation based on command line argument
    let initial_op = match args.operation {
        BooleanOperation::None => BooleanOpState::None,
        BooleanOperation::Subtract => BooleanOpState::Subtract,
        BooleanOperation::Intersect => BooleanOpState::Intersect,
        BooleanOperation::Union => BooleanOpState::Union,
    };
    commands.insert_resource(initial_op);

    // Camera and lighting
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
        transform: Transform::from_xyz(0.0, 1.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    
    // Add a resource to track when STEP is loaded so we can update the primary mesh
    commands.insert_resource(StepModelData {
        step_entity,
        step_handle,
    });
}

// Resource to hold STEP model data while it loads
#[derive(Resource)]
struct StepModelData {
    step_entity: Entity,
    step_handle: Handle<bevy_step_loader::StepAsset>,
}

// System to update the primary mesh with the loaded STEP model when available
fn update_step_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    step_assets: Res<Assets<bevy_step_loader::StepAsset>>,
    step_data: Res<StepModelData>,
    mut has_updated: Local<bool>,
) {
    // Only update once when the STEP asset is loaded
    if !*has_updated {
        if let Some(step_asset) = step_assets.get(&step_data.step_handle) {
            // Update the mesh of the primary entity with the STEP model
            let new_mesh_handle = meshes.add(step_asset.mesh.clone());
            commands.entity(step_data.step_entity).insert(new_mesh_handle);
            *has_updated = true;
        }
    }
}

fn perform_boolean_op_on_loaded_meshes(
    mut op_state: ResMut<BooleanOpState>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    args_resource: Res<ArgsResource>,
) {
    // Only allow keyboard input if the operation wasn't set via command line
    if matches!(args_resource.0.operation, BooleanOperation::None) {
        // A system to trigger boolean operations with keyboard input
        if keyboard_input.just_pressed(KeyCode::Space) {
            *op_state = match *op_state {
                BooleanOpState::None => BooleanOpState::Subtract,
                BooleanOpState::Subtract => BooleanOpState::Intersect,
                BooleanOpState::Intersect => BooleanOpState::Union,
                BooleanOpState::Union => BooleanOpState::None,
            };
        } else if keyboard_input.just_pressed(KeyCode::KeyB) {
            *op_state = BooleanOpState::Subtract; // Boolean cut
        } else if keyboard_input.just_pressed(KeyCode::KeyI) {
            *op_state = BooleanOpState::Intersect; // Intersect
        } else if keyboard_input.just_pressed(KeyCode::KeyU) {
            *op_state = BooleanOpState::Union; // Union
        } else if keyboard_input.just_pressed(KeyCode::KeyN) {
            *op_state = BooleanOpState::None; // No operation
        }
    }
}