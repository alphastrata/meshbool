use bevy::{
    core_pipeline::{experimental::taa::TemporalAntiAliasPlugin, tonemapping::Tonemapping},
    prelude::*,
};
use log;

// A marker component for the operation text UI element
#[derive(Component)]
struct OperationText;

// A marker component for the rotating camera
#[derive(Component)]
struct RotatingCamera;

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

// A resource to hold the current primitive types being used
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
enum PrimitiveShape {
    Cube,
    Sphere,
    Cylinder,
    Tetrahedron,
}

impl Default for PrimitiveShape {
    fn default() -> Self {
        PrimitiveShape::Cube
    }
}

// Separate resources for left and right shape states
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
struct LeftShapeState(PrimitiveShape);

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
struct RightShapeState(PrimitiveShape);

impl Default for LeftShapeState {
    fn default() -> Self {
        LeftShapeState(PrimitiveShape::Cube)
    }
}

impl Default for RightShapeState {
    fn default() -> Self {
        RightShapeState(PrimitiveShape::Sphere)
    }
}

// A resource to hold the current operation mode (boolean ops or trim operations)
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
enum OperationMode {
    Boolean,
    Trim,
}

impl Default for OperationMode {
    fn default() -> Self {
        OperationMode::Boolean
    }
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

// A resource to hold the handles of the entities and meshes used in the manifold demo
#[derive(Resource)]
struct ManifoldDemoHandles {
    shape1: Entity,
    shape2: Entity,
    result: Entity,
    original_mesh1: Handle<Mesh>,
    original_mesh2: Handle<Mesh>,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            bevy_mod_picking::DefaultPickingPlugins,
            TemporalAntiAliasPlugin,
        ))
        // The camera controller works with reactive rendering:
        // .insert_resource(bevy::winit::WinitSettings::desktop_app())
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::srgb(0.15, 0.15, 0.15)))
        .insert_resource(AmbientLight {
            brightness: 0.0,
            ..default()
        })
        // Add the new resources
        .init_resource::<BooleanOpState>()
        .init_resource::<PrimitiveShape>()
        .init_resource::<LeftShapeState>()
        .init_resource::<RightShapeState>()
        .init_resource::<OperationMode>()
        .init_resource::<GeometryStats>()
        .add_systems(Startup, (setup, setup_manifold_demo).chain())
        .add_systems(
            Update,
            (
                // Add the new systems to the update schedule
                cycle_boolean_op,
                cycle_primitive_shape,
                cycle_left_shape,
                cycle_right_shape,
                cycle_operation_mode,
                update_operation_text,
                update_stats_text,
                rotate_camera,
                apply_boolean_op,
            )
                .chain(),
        )
        .run();
}

fn setup(mut commands: Commands) {
    // Add simple lighting to the scene
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

    // Add ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 200.0,
    });

    // Position camera to see the spawned shapes (centered around origin)
    // The shapes are at (-0.5, 1.0, 5.0) and (0.5, 1.0, 5.0), so look at (0, 1, 5) from a good distance
    let cam_trans =
        Transform::from_xyz(0.0, 3.0, 10.0).looking_at(Vec3::new(0.0, 1.0, 5.0), Vec3::Y);

    commands.spawn((
        Camera3dBundle {
            transform: cam_trans,
            tonemapping: Tonemapping::AcesFitted,
            ..default()
        },
        RotatingCamera, // Marker component for camera rotation
    ));

    setup_ui(commands);
}

// This system sets up the two shapes for the boolean operations demo
// Helper function to convert HSL to RGB color
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> Color {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    Color::srgb(r + m, g + m, b + m)
}

// Helper function to generate a diverse set of colors using HSL
fn generate_colors(count: usize) -> Vec<Color> {
    let mut colors = Vec::new();
    for i in 0..count {
        let hue = (i as f32 / count as f32) * 360.0;
        let saturation = 0.8;
        let lightness = 0.6;
        colors.push(hsl_to_rgb(hue, saturation, lightness));
    }
    colors
}

fn setup_manifold_demo(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    left_shape_state: Res<LeftShapeState>,
    right_shape_state: Res<RightShapeState>,
) {
    // Create meshes based on the current shape states
    let mesh1 = meshes.add(create_primitive_mesh(left_shape_state.0));
    let mesh2 = meshes.add(create_primitive_mesh(right_shape_state.0));

    // Generate a diverse set of colors
    let colors = generate_colors(3);
    let material1 = materials.add(colors[0]); // First color for left shape
    let material2 = materials.add(colors[1]); // Second color for right shape
    let result_material = materials.add(colors[2]); // Third color for result

    let shape1 = commands
        .spawn(PbrBundle {
            mesh: mesh1.clone(),
            material: material1,
            transform: Transform::from_xyz(-0.75, 0.0, 0.0),
            ..default()
        })
        .id();

    let shape2 = commands
        .spawn(PbrBundle {
            mesh: mesh2.clone(),
            material: material2,
            transform: Transform::from_xyz(0.75, 0.0, 0.0),
            ..default()
        })
        .id();

    let result = commands
        .spawn(PbrBundle {
            material: result_material,
            // The result mesh is initially hidden
            visibility: Visibility::Hidden,
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        })
        .id();

    commands.insert_resource(ManifoldDemoHandles {
        shape1,
        shape2,
        result,
        original_mesh1: mesh1,
        original_mesh2: mesh2,
    });
}

fn setup_ui(mut commands: Commands) {
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

    let timing_style = TextStyle {
        font_size: 10.0,
        color: Color::WHITE.with_alpha(0.8),
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

            // Bottom right timing information
            parent
                .spawn((NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(20.),
                        right: Val::Px(20.),
                        ..default()
                    },
                    ..default()
                },))
                .with_children(|parent| {
                    parent.spawn((TextBundle::from_section(
                        "Timing: 0ms",
                        timing_style.clone(),
                    ),));
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
                            TextSection::new("N - Cycle left shape\\n", style.clone()),
                            TextSection::new("M - Cycle right shape\\n", style.clone()),
                            TextSection::new("S - Cycle both shapes\\n", style.clone()),
                            TextSection::new(
                                "T - Cycle operation mode (Boolean/Trim)\\n",
                                style.clone(),
                            ),
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
        // Print the current state to the console for debugging
        info!("BooleanOpState changed to: {:?}", *op_state);
    }
}

// This system cycles through the primitive shapes when 'S' key is pressed
fn cycle_primitive_shape(keys: Res<ButtonInput<KeyCode>>, mut shape_state: ResMut<PrimitiveShape>) {
    if keys.just_pressed(KeyCode::KeyS) {
        *shape_state = match *shape_state {
            PrimitiveShape::Cube => PrimitiveShape::Sphere,
            PrimitiveShape::Sphere => PrimitiveShape::Cylinder,
            PrimitiveShape::Cylinder => PrimitiveShape::Tetrahedron,
            PrimitiveShape::Tetrahedron => PrimitiveShape::Cube,
        };
        // Print the current state to the console for debugging
        info!("PrimitiveShape changed to: {:?}", *shape_state);
    }
}

// This system cycles through the left shape when 'N' key is pressed
fn cycle_left_shape(keys: Res<ButtonInput<KeyCode>>, mut left_shape_state: ResMut<LeftShapeState>) {
    if keys.just_pressed(KeyCode::KeyN) {
        left_shape_state.0 = match left_shape_state.0 {
            PrimitiveShape::Cube => PrimitiveShape::Sphere,
            PrimitiveShape::Sphere => PrimitiveShape::Cylinder,
            PrimitiveShape::Cylinder => PrimitiveShape::Tetrahedron,
            PrimitiveShape::Tetrahedron => PrimitiveShape::Cube,
        };
        // Print the current state to the console for debugging
        info!("LeftShape changed to: {:?}", left_shape_state.0);
    }
}

// This system cycles through the right shape when 'M' key is pressed
fn cycle_right_shape(
    keys: Res<ButtonInput<KeyCode>>,
    mut right_shape_state: ResMut<RightShapeState>,
) {
    if keys.just_pressed(KeyCode::KeyM) {
        right_shape_state.0 = match right_shape_state.0 {
            PrimitiveShape::Cube => PrimitiveShape::Sphere,
            PrimitiveShape::Sphere => PrimitiveShape::Cylinder,
            PrimitiveShape::Cylinder => PrimitiveShape::Tetrahedron,
            PrimitiveShape::Tetrahedron => PrimitiveShape::Cube,
        };
        // Print the current state to the console for debugging
        info!("RightShape changed to: {:?}", right_shape_state.0);
    }
}

// This system cycles through operation modes when 'T' key is pressed
fn cycle_operation_mode(keys: Res<ButtonInput<KeyCode>>, mut mode_state: ResMut<OperationMode>) {
    if keys.just_pressed(KeyCode::KeyT) {
        *mode_state = match *mode_state {
            OperationMode::Boolean => OperationMode::Trim,
            OperationMode::Trim => OperationMode::Boolean,
        };
        // Print the current state to the console for debugging
        info!("OperationMode changed to: {:?}", *mode_state);
    }
}

// This system rotates the camera continuously around the origin
fn rotate_camera(time: Res<Time>, mut camera_query: Query<&mut Transform, With<RotatingCamera>>) {
    if let Ok(mut transform) = camera_query.get_single_mut() {
        // Rotate the camera around the origin at double speed
        let time_elapsed = time.elapsed_seconds();
        let rotation_speed = 0.4; // radians per second (doubled from 0.2)

        // Calculate new position in a circular path around the origin
        let radius = 10.0; // Distance from origin
        let x = radius * (rotation_speed * time_elapsed).cos();
        let z = radius * (rotation_speed * time_elapsed).sin();
        let y = 3.0; // Keep the same height

        // Update the camera position
        transform.translation = Vec3::new(x, y, z);

        // Look at the origin (where the shapes are)
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}

// This system updates the UI text to show the current operation and primitive shape
fn update_operation_text(
    op_state: Res<BooleanOpState>,
    shape_state: Res<PrimitiveShape>,
    mode_state: Res<OperationMode>,
    mut query: Query<&mut Text, With<OperationText>>,
) {
    if let Ok(mut text) = query.get_single_mut() {
        let operation_name = match *op_state {
            BooleanOpState::None => "None",
            BooleanOpState::Intersect => "Intersection",
            BooleanOpState::Union => "Union",
            BooleanOpState::Subtract => "Subtraction",
        };

        let shape_name = match *shape_state {
            PrimitiveShape::Cube => "Cube",
            PrimitiveShape::Sphere => "Sphere",
            PrimitiveShape::Cylinder => "Cylinder",
            PrimitiveShape::Tetrahedron => "Tetrahedron",
        };

        let mode_name = match *mode_state {
            OperationMode::Boolean => "Boolean",
            OperationMode::Trim => "Trim",
        };

        text.sections[0].value = format!(
            "Mode: {} | Operation: {} | Shape: {}",
            mode_name, operation_name, shape_name
        );
    }
}

// This system updates the UI text to show current geometry statistics
fn update_stats_text(stats: Res<GeometryStats>, mut query: Query<&mut Text, With<StatsText>>) {
    if stats.is_changed() {
        if let Ok(mut text) = query.get_single_mut() {
            text.sections[0].value =
                format!("Vertices: {} | Edges: {}", stats.vertices, stats.edges);
        }
    }
}

// This system applies the selected boolean operation to the two shapes
#[allow(clippy::too_many_arguments)]
fn apply_boolean_op(
    mut commands: Commands,
    op_state: Res<BooleanOpState>,
    left_shape_state: Res<LeftShapeState>,
    right_shape_state: Res<RightShapeState>,
    mode_state: Res<OperationMode>,
    handles: Res<ManifoldDemoHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut visibility_query: Query<&mut Visibility>,
    mut stats: ResMut<GeometryStats>,
) {
    // Only run if the state has changed
    if !op_state.is_changed() {
        return;
    }

    // If no operation is selected, show the original shapes and hide the result
    if *op_state == BooleanOpState::None {
        if let Ok(mut shape1_vis) = visibility_query.get_mut(handles.shape1) {
            *shape1_vis = Visibility::Visible;
        }
        if let Ok(mut shape2_vis) = visibility_query.get_mut(handles.shape2) {
            *shape2_vis = Visibility::Visible;
        }
        if let Ok(mut result_vis) = visibility_query.get_mut(handles.result) {
            *result_vis = Visibility::Hidden;
        }

        // Recreate the meshes based on the current shape states
        let mesh1 = meshes.add(create_primitive_mesh(left_shape_state.0));
        let mesh2 = meshes.add(create_primitive_mesh(right_shape_state.0));

        commands.entity(handles.shape1).insert(mesh1);
        commands.entity(handles.shape2).insert(mesh2);

        return;
    }

    // Otherwise, hide the original shapes and show the result
    if let Ok(mut shape1_vis) = visibility_query.get_mut(handles.shape1) {
        *shape1_vis = Visibility::Hidden;
    }
    if let Ok(mut shape2_vis) = visibility_query.get_mut(handles.shape2) {
        *shape2_vis = Visibility::Hidden;
    }
    if let Ok(mut result_vis) = visibility_query.get_mut(handles.result) {
        *result_vis = Visibility::Visible;
    }

    // Start timing the entire boolean operation
    let start_time = std::time::Instant::now();

    // Directly create primitive manifolds instead of converting from Bevy meshes
    let mesh_conversion_start = std::time::Instant::now();

    // Create different primitive combinations based on the current left and right shape states
    // Position them to actually intersect for meaningful boolean operations
    let primitive1 = match left_shape_state.0 {
        PrimitiveShape::Cube => manifold_rs::Manifold::cube(1.0, 1.0, 1.0),
        PrimitiveShape::Sphere => manifold_rs::Manifold::sphere(1.0, 64),
        PrimitiveShape::Cylinder => manifold_rs::Manifold::sphere(1.0, 64), // Fallback to sphere
        PrimitiveShape::Tetrahedron => create_tetrahedron(1.0),
    };

    let primitive2 = match right_shape_state.0 {
        PrimitiveShape::Cube => manifold_rs::Manifold::cube(1.0, 1.0, 1.0),
        PrimitiveShape::Sphere => manifold_rs::Manifold::sphere(1.0, 64),
        PrimitiveShape::Cylinder => manifold_rs::Manifold::sphere(0.8, 48), // Smaller sphere
        PrimitiveShape::Tetrahedron => create_tetrahedron(0.8),
    };

    // Move primitive2 slightly to create intersection
    let primitive2 = primitive2.translate(0.5, 0.0, 0.0);

    let (primitive1, primitive2) = (primitive1, primitive2);

    let mesh_conversion_time = mesh_conversion_start.elapsed();

    // Perform the operation based on the current mode (boolean or trim)
    let boolean_op_start = std::time::Instant::now();
    let result_manifold = if *mode_state == OperationMode::Boolean {
        match *op_state {
            BooleanOpState::Intersect => {
                primitive1.boolean_op(&primitive2, manifold_rs::BooleanOp::Intersection)
            }
            BooleanOpState::Union => {
                primitive1.boolean_op(&primitive2, manifold_rs::BooleanOp::Union)
            }
            BooleanOpState::Subtract => {
                primitive1.boolean_op(&primitive2, manifold_rs::BooleanOp::Difference)
            }
            BooleanOpState::None => unreachable!(),
        }
    } else {
        // For trim operations, we'll trim one shape with a plane
        // Create a plane that cuts through the middle
        let x = 0.0; // Normal vector X component
        let y = 1.0; // Normal vector Y component (horizontal plane)
        let z = 0.0; // Normal vector Z component
        let offset = 0.0; // Offset from origin

        // Apply the trim operation to the first primitive
        primitive1.trim_by_plane(x, y, z, offset)
    };
    let boolean_op_time = boolean_op_start.elapsed();

    // Convert the resulting Manifold back to a Bevy mesh
    let mesh_conversion_back_start = std::time::Instant::now();
    let result_mesh = manifold_to_bevy_mesh(result_manifold);
    let result_vertex_count = result_mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .map(|values| {
            if let bevy::render::mesh::VertexAttributeValues::Float32x3(positions) = values {
                positions.len()
            } else {
                0
            }
        })
        .unwrap_or(0);

    // Update the stats resource with current geometry information
    stats.vertices = result_vertex_count;
    stats.edges = 0; // Edges not easily available from manifold-rs

    let result_mesh_handle = meshes.add(result_mesh);
    let mesh_conversion_back_time = mesh_conversion_back_start.elapsed();

    // Update the mesh of the result entity
    let update_entity_start = std::time::Instant::now();
    commands.entity(handles.result).insert(result_mesh_handle);
    let update_entity_time = update_entity_start.elapsed();

    let total_time = start_time.elapsed();

    // Log the timing information and vertex counts
    log::info!("Boolean operation timing:");
    log::info!("  Total time: {:.2?}", total_time);
    log::info!("  Mesh conversion: {:.2?}", mesh_conversion_time);
    log::info!("  Transform: 0ns"); // No transform operations in this approach
    log::info!("  Boolean op: {:.2?}", boolean_op_time);
    log::info!("  Mesh conversion back: {:.2?}", mesh_conversion_back_time);
    log::info!("  Update entity: {:.2?}", update_entity_time);

    // Log geometric information - using the actual primitive counts
    let primitive1_mesh_info = primitive1.to_mesh();
    let primitive2_mesh_info = primitive2.to_mesh();
    let original_vertices1 = primitive1_mesh_info.vertices().len();
    let original_vertices2 = primitive2_mesh_info.vertices().len();

    log::info!("  Original cube vertices: {}", original_vertices1);
    log::info!("  Original sphere vertices: {}", original_vertices2);
    log::info!("  Result mesh vertices: {}", result_vertex_count);

    // Assertion: Subtraction should normally reduce vertex count compared to union
    if *op_state == BooleanOpState::Subtract {
        log::info!("  Subtraction operation: checking vertex reduction");
        if result_vertex_count > 0
            && result_vertex_count < (original_vertices1 + original_vertices2)
        {
            log::info!(
                "  ✓ Subtraction resulted in vertex reduction ({} -> {})",
                original_vertices1 + original_vertices2,
                result_vertex_count
            );
        } else if result_vertex_count == 0 {
            log::warn!("  ! Subtraction resulted in empty mesh");
        } else {
            log::warn!("  ! Subtraction did not reduce vertex count as expected");
        }
    } else if *op_state == BooleanOpState::Union {
        log::info!(
            "  Union operation: {} -> {}",
            original_vertices1 + original_vertices2,
            result_vertex_count
        );
    } else if *op_state == BooleanOpState::Intersect {
        log::info!(
            "  Intersection operation: {} -> {}",
            original_vertices1 + original_vertices2,
            result_vertex_count
        );
    }
}

// Creates a tetrahedron as a manifold-rs Manifold
fn create_tetrahedron(size: f64) -> manifold_rs::Manifold {
    // Define the vertices of a tetrahedron
    // A regular tetrahedron with edge length 'size'
    let vertices = vec![
        0.0f32,
        0.0,
        (size * (2.0f64.sqrt() / 3.0)) as f32, // Top vertex
        (-size / 2.0) as f32,
        (-size / (2.0 * 3.0f64.sqrt())) as f32,
        0.0, // Base vertex 1
        (size / 2.0) as f32,
        (-size / (2.0 * 3.0f64.sqrt())) as f32,
        0.0, // Base vertex 2
        0.0,
        (size / (3.0f64.sqrt())) as f32,
        0.0, // Base vertex 3
    ];

    // Define the indices for the tetrahedron faces (4 triangles)
    let indices = vec![
        0, 1, 2, // Face 1
        0, 2, 3, // Face 2
        0, 3, 1, // Face 3
        1, 3, 2, // Face 4
    ];

    // Create a mesh and then a manifold from it
    let mesh = manifold_rs::Mesh::new(&vertices, &indices);
    manifold_rs::Manifold::from_mesh(mesh)
}

// Converts a Bevy mesh to a manifold-rs Manifold in its local space
fn bevy_mesh_to_manifold(mesh: &Mesh) -> Option<manifold_rs::Manifold> {
    // Get positions
    let positions = if let Some(positions) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        match positions {
            bevy::render::mesh::VertexAttributeValues::Float32x3(pos) => pos,
            _ => {
                log::warn!("Position attribute is not Float32x3");
                return None;
            }
        }
    } else {
        log::warn!("No position attribute found");
        return None;
    };

    // Get indices
    let indices = if let Some(indices) = mesh.indices() {
        match indices {
            bevy::render::mesh::Indices::U32(indices_vec) => indices_vec.clone(),
            bevy::render::mesh::Indices::U16(indices_vec) => {
                // Convert u16 to u32 indices
                indices_vec.iter().map(|&i| i as u32).collect()
            }
            _ => {
                log::warn!("Indices are not U32 or U16");
                return None;
            }
        }
    } else {
        log::warn!("No indices found");
        return None;
    };

    // Convert vertices to the format expected by manifold-rs
    let vertices_f32: Vec<f32> = positions.iter().flat_map(|p| [p[0], p[1], p[2]]).collect();

    log::debug!(
        "Mesh data: {} vertices, {} indices",
        positions.len(),
        indices.len()
    );

    let mesh = manifold_rs::Mesh::new(&vertices_f32, &indices);
    let manifold = manifold_rs::Manifold::from_mesh(mesh);
    let mesh_info = manifold.to_mesh();
    log::debug!(
        "Created manifold with {} vertices",
        mesh_info.vertices().len()
    );
    Some(manifold)
}

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

// Helper function to create primitive meshes based on the shape type
fn create_primitive_mesh(shape: PrimitiveShape) -> Mesh {
    match shape {
        PrimitiveShape::Cube => Mesh::from(Cuboid::new(1.5, 1.5, 1.5)),
        PrimitiveShape::Sphere => Mesh::from(Sphere::new(1.0)),
        PrimitiveShape::Cylinder => Mesh::from(Cylinder::new(0.5, 1.5)),
        PrimitiveShape::Tetrahedron => Mesh::from(Sphere::new(1.0)), // Fallback to sphere for tetrahedron
    }
}
