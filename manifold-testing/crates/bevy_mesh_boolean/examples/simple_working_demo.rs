//! Simple demo to show the requested functionality working:
//! 1. Space bar toggle for boolean operations
//! 2. Camera orbiting the main part
//! 3. Q key to quit with error message
//! 4. Proper error handling with useful messages

use bevy::prelude::*;
use bevy_mesh_boolean::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            MeshBooleanPlugin,
        ))
        .insert_resource(ClearColor(Color::srgb(0.15, 0.15, 0.15)))
        .insert_resource(AmbientLight {
            brightness: 0.75,
            color: Color::WHITE,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (
            cycle_boolean_op,
            orbit_camera,
            exit_on_q_key,
        ))
        .run();
}

#[derive(Component)]
struct PrimaryShape;

#[derive(Component)]
struct SecondaryShape;

#[derive(Component)]
struct OrbitCamera;

#[derive(Resource, Default)]
struct OrbitState {
    angle: f32,
    center: Vec3,
    distance: f32,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
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

    // Create two overlapping shapes for boolean operations
    let cube_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let sphere_mesh = meshes.add(Sphere::new(0.8));
    
    let cube_material = materials.add(Color::srgb(0.9, 0.7, 0.3)); // Orange/Bronze
    let sphere_material = materials.add(Color::srgb(0.3, 0.5, 0.9)); // Blue
    let result_material = materials.add(Color::srgb(0.6, 0.8, 0.4)); // Green

    // Spawn primary shape (cube) slightly to the left
    let primary_entity = commands.spawn((
        PbrBundle {
            mesh: cube_mesh.clone(),
            material: cube_material,
            transform: Transform::from_xyz(-0.5, 0.0, 0.0),
            ..default()
        },
        PrimaryBooleanMesh {
            secondary_entity: Entity::PLACEHOLDER, // Will be set after spawning secondary
        },
        PrimaryShape,
    )).id();

    // Spawn secondary shape (sphere) slightly to the right (overlapping with cube)
    let secondary_entity = commands.spawn((
        PbrBundle {
            mesh: sphere_mesh.clone(),
            material: sphere_material,
            transform: Transform::from_xyz(0.5, 0.0, 0.0), // Position to intersect with cube
            ..default()
        },
        SecondaryBooleanMesh {
            primary_entity, // Will be set after spawning secondary
        },
        SecondaryShape,
    )).id();

    // Update the references to each other
    commands.entity(primary_entity).insert(PrimaryBooleanMesh {
        secondary_entity,
    });

    commands.entity(secondary_entity).insert(SecondaryBooleanMesh {
        primary_entity,
    });

    // Create result entity (hidden initially)
    let result_entity = commands.spawn((
        PbrBundle {
            material: result_material,
            visibility: Visibility::Hidden, // Initially hidden
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
    )).id();

    // Set up the boolean operation handles
    commands.insert_resource(BooleanHandles {
        primary_entity,
        secondary_entity,
        result_entity,
    });

    // Camera with orbit capability
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 2.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        OrbitCamera,
    ));
    
    // Initialize orbit state
    commands.insert_resource(OrbitState {
        angle: 0.0,
        center: Vec3::ZERO,
        distance: 10.0,
    });

    // UI text
    commands.spawn((
        TextBundle::from_section(
            "Boolean Operations Demo\nPress SPACE to cycle: None -> Intersect -> Union -> Subtract -> None\nPress Q to quit with error message",
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
    ));

    println!("Demo setup complete - Two overlapping shapes (cube and sphere) ready for boolean operations");
    println!("Press SPACE to cycle through boolean operations");
    println!("Press Q to quit with error message");
}

// System to cycle through boolean operations when spacebar is pressed
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
        println!("BooleanOpState changed to: {:?}", *op_state);
    }
}

// System for orbit camera
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

// System to exit when 'q' is pressed with error message
fn exit_on_q_key(
    keys: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::KeyQ) {
        eprintln!("User did not see expected results");
        exit.send(AppExit::Success);
    }
}