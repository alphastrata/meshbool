//! Simple demo that loads a STEP file and shows all three meshes simultaneously
//! LHS (primary), RHS (secondary), and Result of boolean operation
//! With space bar toggle and camera orbiting

use bevy::prelude::*;
use bevy_mesh_boolean::*;
use bevy_step_loader::*;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <step_file.step>", args[0]);
        eprintln!("Example: {} assets/LN_032.step", args[0]);
        std::process::exit(1);
    }
    
    let step_file_path = &args[1];
    
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
        .insert_resource(StepFilePath(step_file_path.clone()))
        .insert_resource(BooleanOpState::None)
        .add_systems(Startup, (setup_scene, setup_ui))
        .add_systems(Update, (
            load_step_and_setup_meshes,
            cycle_boolean_op,
            orbit_camera,
            exit_on_q_key,
        ))
        .run();
}

#[derive(Resource)]
struct StepFilePath(String);

#[derive(Component)]
struct PrimaryShape;

#[derive(Component)]
struct SecondaryShape;

#[derive(Component)]
struct ResultShape;

#[derive(Component)]
struct OrbitCamera;

#[derive(Resource, Default)]
struct OrbitState {
    angle: f32,
    center: Vec3,
    distance: f32,
}

#[derive(Component)]
struct OperationText;

fn setup_scene(
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

    // Spawn camera with orbit capability
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
}

fn setup_ui(mut commands: Commands) {
    // UI text to show current operation
    commands.spawn((
        TextBundle::from_section(
            "Boolean Operations Demo\nPress SPACE to cycle operations\nPress Q to quit with error message",
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
        println!("Boolean operation changed to: {:?}", *op_state);
    }
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

fn exit_on_q_key(
    keys: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::KeyQ) {
        eprintln!("User did not see expected results");
        exit.send(AppExit::Success);
    }
}