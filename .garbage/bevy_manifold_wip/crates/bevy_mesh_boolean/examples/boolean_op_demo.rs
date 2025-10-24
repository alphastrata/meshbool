// In examples/boolean_op_demo.rs
use bevy::prelude::*;
use bevy_mesh_boolean::*; // Import our new plugin

// Converts a manifold-rs Manifold to a Bevy mesh
fn manifold_to_bevy_mesh(manifold: manifold_rs::Manifold) -> Mesh {
    let mesh = manifold.to_mesh();

    let vertices = mesh.vertices();
    let indices = mesh.indices();

    match mesh.num_props() {
        3 => {
            // Vertex without normals
            let vertices: Vec<[f32; 3]> = vertices.chunks(3).map(|c| [c[0], c[1], c[2]]).collect();

            Mesh::new(
                bevy::render::mesh::PrimitiveTopology::TriangleList,
                bevy::render::render_asset::RenderAssetUsages::all(),
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
            .with_inserted_indices(bevy::render::mesh::Indices::U32(indices))
            .with_duplicated_vertices()
            .with_computed_flat_normals()
        }
        6 => {
            // Vertex with normals
            let normals: Vec<[f32; 3]> = vertices.chunks(6).map(|c| [c[3], c[4], c[5]]).collect();
            let vertices: Vec<[f32; 3]> = vertices.chunks(6).map(|c| [c[0], c[1], c[2]]).collect();

            Mesh::new(
                bevy::render::mesh::PrimitiveTopology::TriangleList,
                bevy::render::render_asset::RenderAssetUsages::all(),
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
            .with_inserted_indices(bevy::render::mesh::Indices::U32(indices))
        }
        num_props => panic!("Invalid property count {num_props}"),
    }
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MeshBooleanPlugin)) // Add our plugin
        .add_systems(Startup, setup)
        .add_systems(Update, (cycle_boolean_op, update_operation_text))
        .run();
}

#[derive(Component)]
struct OperationText;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create Bevy-compatible meshes that can be converted to manifold-rs
    eprintln!("[SETUP] Creating Bevy-compatible meshes...");
    
    // Create a cube as the primary mesh using Bevy primitives (can be converted to manifold)
    let cube_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let cube_material = materials.add(Color::srgb(0.8, 0.7, 0.6));
    eprintln!("[SETUP] Created cube mesh with Bevy Cuboid");

    // Create a sphere as the secondary mesh using Bevy primitives (can be converted to manifold)
    let sphere_mesh = meshes.add(Sphere::new(0.8));
    let sphere_material = materials.add(Color::srgb(0.6, 0.7, 0.8));
    eprintln!("[SETUP] Created sphere mesh with Bevy Sphere");

    // Spawn the primary cube
    let primary_entity = commands
        .spawn(PbrBundle {
            mesh: cube_mesh,
            material: cube_material,
            transform: Transform::from_xyz(-0.4, 0.0, 0.0),
            ..default()
        })
        .insert(PrimaryBooleanMesh {
            secondary_entity: Entity::PLACEHOLDER, // Will be set after spawning secondary
        })
        .id();
    eprintln!("[SETUP] Spawned primary cube entity: {:?}", primary_entity);

    // Spawn the secondary sphere
    let secondary_entity = commands
        .spawn(PbrBundle {
            mesh: sphere_mesh,
            material: sphere_material,
            transform: Transform::from_xyz(0.4, 0.0, 0.0),
            ..default()
        })
        .insert(SecondaryBooleanMesh {
            primary_entity: primary_entity,
        })
        .id();
    eprintln!("[SETUP] Spawned secondary sphere entity: {:?}", secondary_entity);

    // Create the result entity (initially hidden)
    let result_entity = commands
        .spawn(PbrBundle {
            material: materials.add(Color::srgb(0.5, 0.8, 0.5)), // Different color for result
            visibility: Visibility::Hidden, // Initially hidden
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        })
        .id();
    eprintln!("[SETUP] Created result entity: {:?}", result_entity);

    // Update the primary entity to reference the secondary
    commands.entity(primary_entity).insert(PrimaryBooleanMesh {
        secondary_entity: secondary_entity,
    });

    // Insert the handles resource
    commands.insert_resource(BooleanHandles {
        primary_entity,
        secondary_entity,
        result_entity,
    });
    eprintln!("[SETUP] Inserted BooleanHandles resource");

    // Add camera and lighting
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 3.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
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
    
    eprintln!("[SETUP] Completed setup with Bevy-compatible meshes");
}

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