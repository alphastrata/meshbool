use bevy::prelude::*;
use meshbool::{cube, get_mesh_gl};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create a cube using meshbool
    let cube_impl = cube(nalgebra::Vector3::new(2.0, 2.0, 2.0), true);
    println!("✓ Created meshbool cube with {} triangles", cube_impl.num_tri());
    
    // Convert to MeshGL
    let mesh_gl = get_mesh_gl(&cube_impl, 0);
    println!("✓ Converted to MeshGL: {} vertices, {} triangles", 
             mesh_gl.vert_properties.len() / mesh_gl.num_prop as usize,
             mesh_gl.tri_verts.len() / 3);
    
    // Convert to Bevy mesh
    let bevy_mesh = meshgl_to_bevy_mesh(&mesh_gl);
    let mesh_handle = meshes.add(bevy_mesh);
    
    println!("✓ Converted to Bevy mesh successfully");
    
    // Spawn in scene
    commands.spawn((
        Mesh3d(mesh_handle),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.1, 0.1))),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    
    // Add lighting and camera
    commands.spawn((
        PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 3.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    
    println!("✅ Meshbool-Bevy integration verified!");
}

/// Convert meshbool MeshGL to Bevy Mesh
fn meshgl_to_bevy_mesh(mesh_gl: &meshbool::MeshGL) -> Mesh {
    let mut bevy_mesh = Mesh::new(
        bevy_mesh::PrimitiveTopology::TriangleList,
        bevy_asset::RenderAssetUsages::default()
    );
    
    // Extract vertex data
    let num_verts = mesh_gl.vert_properties.len() / mesh_gl.num_prop as usize;
    let mut positions = Vec::with_capacity(num_verts);
    
    for i in 0..num_verts {
        let offset = i * mesh_gl.num_prop as usize;
        positions.push([
            mesh_gl.vert_properties[offset],
            mesh_gl.vert_properties[offset + 1], 
            mesh_gl.vert_properties[offset + 2]
        ]);
    }
    
    // Extract indices
    let indices: Vec<u32> = mesh_gl.tri_verts.clone();
    
    // Insert data into Bevy mesh
    bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    bevy_mesh.insert_indices(bevy_mesh::Indices::U32(indices));
    
    bevy_mesh
}