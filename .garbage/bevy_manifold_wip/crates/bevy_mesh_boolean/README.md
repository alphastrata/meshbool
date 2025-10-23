# bevy_mesh_boolean

A Bevy plugin that provides capabilities for performing boolean operations (intersection, union, subtraction) on Bevy meshes.

## Features

- Perform boolean operations on Bevy meshes (intersect, union, subtract)
- Robust geometric computation using manifold-rs library
- Real-time visualization of boolean operation results
- Support for complex mesh topologies
- Integration with STEP file loading via bevy_step_loader

## Boolean Operations

### Supported Operations

- **Intersection**: Creates a mesh representing the overlapping volume between two meshes
- **Union**: Combines two meshes into a single unified mesh
- **Subtraction**: Cuts one mesh from another (difference operation)

### Operation Controls

- **Space**: Cycle through boolean operations (None → Intersect → Union → Subtract → None)
- **B**: Set operation to Subtract (Cut)
- **I**: Set operation to Intersect
- **U**: Set operation to Union
- **N**: Set operation to None

## Usage

### Basic Usage

```toml
[dependencies]
bevy_mesh_boolean = { path = "path/to/crates/bevy_mesh_boolean" }
```

```rust
use bevy::prelude::*;
use bevy_mesh_boolean::MeshBooleanPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MeshBooleanPlugin))
        .run();
}
```

### Setting Up Boolean Operations

To perform boolean operations between two meshes:

1. Spawn your primary and secondary meshes with appropriate components
2. Set up a result entity to receive the operation result
3. Control the operation through the `BooleanOpState` resource

### Example Setup

```rust
use bevy::prelude::*;
use bevy_mesh_boolean::*;

fn setup_boolean_operation(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create primary mesh (cube)
    let primary_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let primary_material = materials.add(Color::srgb(0.8, 0.7, 0.6));
    
    let primary_entity = commands
        .spawn(PbrBundle {
            mesh: primary_mesh,
            material: primary_material,
            transform: Transform::from_xyz(-0.5, 0.0, 0.0),
            ..default()
        })
        .insert(PrimaryBooleanMesh {
            secondary_entity: Entity::PLACEHOLDER,
        })
        .id();

    // Create secondary mesh (sphere)
    let secondary_mesh = meshes.add(Sphere::new(0.8));
    let secondary_material = materials.add(Color::srgb(0.6, 0.7, 0.8));
    
    let secondary_entity = commands
        .spawn(PbrBundle {
            mesh: secondary_mesh,
            material: secondary_material,
            transform: Transform::from_xyz(0.5, 0.0, 0.0),
            ..default()
        })
        .insert(SecondaryBooleanMesh {
            primary_entity,
        })
        .id();

    // Update primary entity reference
    commands.entity(primary_entity).insert(PrimaryBooleanMesh {
        secondary_entity,
    });

    // Create result entity
    let result_entity = commands
        .spawn(PbrBundle {
            material: materials.add(Color::srgb(0.9, 0.5, 0.5)),
            visibility: Visibility::Hidden,
            ..default()
        })
        .id();

    // Store handles for the boolean operation system
    commands.insert_resource(BooleanHandles {
        primary_entity,
        secondary_entity,
        result_entity,
    });
}
```

## Integration with STEP Files

The plugin works seamlessly with STEP files loaded via `bevy_step_loader`:

```rust
// Load a STEP file
let step_handle: Handle<StepAsset> = asset_server.load("model.step");

// When the STEP file loads, you can use its mesh for boolean operations
if let Some(step_asset) = step_assets.get(&step_handle) {
    let step_mesh_handle = meshes.add(step_asset.mesh.clone());
    // Then use step_mesh_handle in boolean operations
}
```

## Examples

### boolean_op_demo
Demonstrates boolean operations between basic geometric shapes (cube and sphere) with real-time visualization and keyboard controls.

### composite_example
Loads a STEP file and performs boolean operations on it using a cube, demonstrating integration with bevy_step_loader.

## Dependencies

- `bevy` - The Bevy game engine
- `manifold-rs` - For robust boolean mesh operations

## License

This project is licensed under your chosen license. See the LICENSE file for details.