// In examples/load_step_file.rs
use bevy::prelude::*;
use bevy_step_loader::{StepAsset, StepPlugin}; // Import our plugin and asset type

fn main() {
    // For now, we'll hardcode the file path as an argument
    // In a real application you'd use command line args
    let step_file = std::env::args().nth(1).unwrap_or_else(|| "real_parts/multifeature.step".to_string());
    
    App::new()
        .add_plugins((DefaultPlugins.set(ImagePlugin::default_nearest()), StepPlugin)) // Add our plugin
        .insert_resource(StepFileResource(step_file))
        .add_systems(Startup, setup)
        .add_systems(Update, update_step_mesh)
        .run();
}

#[derive(Resource)]
struct StepFileResource(String);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    step_file_resource: Res<StepFileResource>,
) {
    let step_file = &step_file_resource.0;
    
    // Load the STEP file specified via command line
    let step_handle: Handle<StepAsset> = asset_server.load(step_file);
    
    // Material for the STEP model
    let step_material = materials.add(StandardMaterial::from(Color::srgb(0.8, 0.7, 0.6)));
    
    // Create a placeholder mesh initially
    let placeholder_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    
    // Spawn an entity with the placeholder that we'll update when the STEP file loads
    commands.spawn((
        PbrBundle {
            mesh: placeholder_mesh,
            material: step_material,
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        StepModel { handle: step_handle },
    ));

    // Camera and light setup
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
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

#[derive(Component)]
struct StepModel {
    handle: Handle<StepAsset>,
}

fn update_step_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    step_assets: Res<Assets<StepAsset>>,
    step_models: Query<(Entity, &StepModel)>,
) {
    for (entity, step_model) in step_models.iter() {
        if let Some(step_asset) = step_assets.get(&step_model.handle) {
            // Update the mesh of the entity with the loaded STEP model
            let new_mesh_handle = meshes.add(step_asset.mesh.clone());
            commands.entity(entity).insert(new_mesh_handle);
            // Remove the StepModel component since we've loaded it
            commands.entity(entity).remove::<StepModel>();
        }
    }
}