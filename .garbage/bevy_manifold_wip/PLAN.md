# RULES:
- rust must compile
- composable code
- correctly crate isolated code
- reference the ./old/**/*.rs content when in doubt.
- use idiomatic rust
- use iterators
- avoid bringing in dpeendencies!
- consider anyhow vs a custom error type carefully! anyhow is usually for binaries, but our plugins are techincally libraries!
- my notes below are not perfect, because _I_ am not perfect.

## **Overall Goal**

Our objective is to refactor a monolithic Bevy application into a well-structured workspace containing two distinct and reusable plugins:

1.  **`bevy_mesh_boolean`**: A plugin that provides capabilities for performing boolean operations (intersection, union, subtraction) on Bevy meshes.
2.  **`bevy_step_loader`**: A plugin that introduces a custom asset loader for `.step` and `.stp` files, enabling Bevy to load them as assets.

These plugins will be designed to be completely independent, allowing them to be used individually or together in any Bevy project.

## **Project Structure**

We will organize our project as a Cargo workspace to manage the multiple crates:

```
/bevvy_plugins_workspace
|--/crates
|  |--/bevy_mesh_boolean
|  |  |--/examples
|  |  |  |-- boolean_op_demo.rs
|  |  |--/src
|  |  |  |-- lib.rs
|  |--/bevy_step_loader
|  |  |--/examples
|  |  |  |-- load_step_file.rs
|  |  |--/src
|  |  |  |-- lib.rs
|-- Cargo.toml
```

### **`Cargo.toml` at Workspace Root**

This file will define the workspace and its members.

```toml
[workspace]
members = [
    "crates/bevy_mesh_boolean",
    "crates/bevy_step_loader",
]

# We also want all our deps controlled (versions etc) by the workspace toml
[dependencies]
bevy = "0.14.2"
bevy_editor_cam = "0.4"
manifold-rs = "0.6.2"
bevy_mod_picking = { version = "0.20.0", default-features = false, features = [
    "backend_raycast",
] }
log = "0.4"
env_logger = "0.11"

step = { git = "https://github.com/alphastrata/foxtrot.git", branch = "modernise", package = "step", features = [
    "rayon",
] }
triangulate = { git = "https://github.com/alphastrata/foxtrot.git", branch = "modernise", package = "triangulate", features = [
    "rayon",
    "wgpu",
] }

# all crates should use the xyz = {workspace = true, ...otherfields....} syntax.
```

---

## **Part 1: Creating the `bevy_mesh_boolean` Plugin**

This plugin will encapsulate all the logic related to performing boolean operations on meshes.

### **Step 1.1: Setting up the Crate**

1.  Create the directory `crates/bevy_mesh_boolean`.
2.  Inside this directory, create a `Cargo.toml` file with the following content:

    ```toml
    [package]
    name = "bevy_mesh_boolean"
    version = "0.1.0"
    edition # get from workspace.

    [dependencies]
    bevy.workspace = true
    manifold-rs.workspace=true
...
    ```

3.  Create the `src` directory with a `lib.rs` file inside.

### **Step 1.2: Defining the Public API in `lib.rs`**

We will define the core components of our plugin in `src/lib.rs`.

```rust
use bevy::prelude::*;

// The core plugin struct
pub struct MeshBooleanPlugin;

impl Plugin for MeshBooleanPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BooleanOpState>()
            .add_systems(Update, apply_boolean_op);
    }
}

// Resource to control the boolean operation
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum BooleanOpState {
    #[default]
    None,
    Intersect,
    Union,
    Subtract,
}

// Marker component for entities involved in a boolean operation
#[derive(Component)]
pub struct BooleanMesh {
    pub op: BooleanOp,
    pub secondary_entity: Entity,
}

// Enum to define the role in the boolean operation
#[derive(Clone, Copy)]
pub enum BooleanOp {
    Primary,
    Secondary,
}


// The system that applies the boolean operation
fn apply_boolean_op(
    mut commands: Commands,
    query: Query<(Entity, &BooleanMesh, &Handle<Mesh>, &GlobalTransform)>,
    meshes: Res<Assets<Mesh>>,
    op_state: Res<BooleanOpState>,
) {
    if *op_state == BooleanOpState::None {
        return;
    }

    let mut primary_mesh: Option<(Handle<Mesh>, GlobalTransform)> = None;
    let mut secondary_mesh: Option<(Handle<Mesh>, GlobalTransform)> = None;
    let mut primary_entity = None;

    for (entity, boolean_mesh_comp, mesh_handle, transform) in query.iter() {
        match boolean_mesh_comp.op {
            BooleanOp::Primary => {
                primary_mesh = Some((mesh_handle.clone(), *transform));
                primary_entity = Some(entity);
            }
            BooleanOp::Secondary => {
                if boolean_mesh_comp.secondary_entity == primary_entity.unwrap() {
                    secondary_mesh = Some((mesh_handle.clone(), *transform));
                }
            }
        }
    }
    if let (Some((primary_handle, primary_transform)), Some((secondary_handle, secondary_transform))) = (primary_mesh, secondary_mesh) {
        if let (Some(primary_bevy_mesh), Some(secondary_bevy_mesh)) = (meshes.get(&primary_handle), meshes.get(&secondary_handle)) {

            // Convert Bevy meshes to manifold-rs manifolds
            let mut primary_manifold = bevy_mesh_to_manifold(primary_bevy_mesh).unwrap();
            let mut secondary_manifold = bevy_mesh_to_manifold(secondary_bevy_mesh).unwrap();
            
            // Apply transformations
            primary_manifold = primary_manifold.transform(primary_transform.compute_matrix().to_cols_array_2d());
            secondary_manifold = secondary_manifold.transform(secondary_transform.compute_matrix().to_cols_array_2d());


            // Perform boolean operation
            let result_manifold = match *op_state {
                BooleanOpState::Intersect => primary_manifold.boolean_op(&secondary_manifold, manifold_rs::BooleanOp::Intersection),
                BooleanOpState::Union => primary_manifold.boolean_op(&secondary_manifold, manifold_rs::BooleanOp::Union),
                BooleanOpState::Subtract => primary_manifold.boolean_op(&secondary_manifold, manifold_rs::BooleanOp::Difference),
                BooleanOpState::None => unreachable!(),
            };

            // Convert back to Bevy mesh and update the primary entity
            let result_bevy_mesh = manifold_to_bevy_mesh(result_manifold);
            let result_mesh_handle = meshes.add(result_bevy_mesh);
            commands.entity(primary_entity.unwrap()).insert(result_mesh_handle);
        }
    }
}

// Utility functions for mesh conversion (to be included in the library)

pub fn bevy_mesh_to_manifold(mesh: &Mesh) -> Option<manifold_rs::Manifold> {
    // ... implementation from the original code ...
}

pub fn manifold_to_bevy_mesh(manifold: manifold_rs::Manifold) -> Mesh {
    // ... implementation from the original code ...
}

```

### **Step 1.3: Creating an Example**

To demonstrate how to use the `bevy_mesh_boolean` plugin, we'll create an example.

1.  Create the directory `crates/bevy_mesh_boolean/examples`.
2.  Inside, create a file named `boolean_op_demo.rs`. This file will contain the UI, camera controls, and scene setup from the original `main.rs`.

```rust
// In examples/boolean_op_demo.rs
use bevy::prelude::*;
use bevy_mesh_boolean::*; // Import our new plugin

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MeshBooleanPlugin)) // Add our plugin
        .add_systems(Startup, setup)
        .add_systems(Update, (cycle_boolean_op, update_operation_text))
        .run();
}

// ... include setup, UI, camera rotation, and other systems from the original `main.rs` ...
// ... make sure to modify the setup to use the `BooleanMesh` component ...
```

---

## **Part 2: Creating the `bevy_step_loader` Plugin**

This plugin will be responsible for loading `.step` files.

### **Step 2.1: Setting up the Crate**

1.  Create the directory `crates/bevy_step_loader`.
2.  Inside, create a `Cargo.toml` file:

    ```toml
    [package]
    name = "bevy_step_loader"
    version = "0.1.0"
    edition, deps etc from the workspace toml.
    ```

3.  Create the `src` directory with a `lib.rs` file.

### **Step 2.2: Implementing the Asset Loader in `lib.rs`**

The `lib.rs` will contain the plugin definition, the custom asset type, and the asset loader.

```rust
// In crates/bevy_step_loader/src/lib.rs

use bevy::{
    asset::{Asset, AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypePath,
    render::mesh::{Indices, Mesh},
    utils::BoxedFuture,
};

// The plugin to register the asset and loader
pub struct StepPlugin;

impl Plugin for StepPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<StepAsset>()
           .register_asset_loader(StepLoader);
    }
}

// The asset representing a STEP file
#[derive(Asset, TypePath, Debug, Clone)]
pub struct StepAsset {
    pub mesh: Mesh,
}

// The loader for STEP files
#[derive(Default)]
pub struct StepLoader;

impl AssetLoader for StepLoader {
    type Asset = StepAsset;
    type Settings = ();
    type Error = anyhow::Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            
            // Load STEP data using foxtrot
            let step_data = foxtrot::load_step_from_slice(&bytes)?;
            
            // Convert to Bevy mesh
            let mesh = convert_to_bevy_mesh(step_data.mesh())?;
            
            Ok(StepAsset { mesh })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["step", "stp"]
    }
}

fn convert_to_bevy_mesh(step_mesh: &foxtrot::Mesh) -> Result<Mesh, anyhow::Error> {
    // ... implementation from the original code ...
}

```

### **Step 2.3: Creating an Example**

1.  Create the `examples` directory inside `crates/bevy_step_loader`.
2.  Create a file named `load_step_file.rs`.

```rust
// In examples/load_step_file.rs
use bevy::prelude::*;
use bevy_step_loader::StepPlugin; // Import our plugin

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, StepPlugin)) // Add our plugin
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Load a STEP file
    let step_handle: Handle<bevy_step_loader::StepAsset> = asset_server.load("path/to/your/model.step");

    commands.spawn(PbrBundle {
        // We need to get the mesh from our custom asset
        mesh: step_handle.mesh.clone(), 
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        ..default()
    });

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
```

---

## **Part 3: Using the Plugins Together**

To demonstrate the composability of our new plugins, we can create a new example in the workspace root or within one of the crates that uses both plugins.

1.  Create a new `examples` directory at the root of the workspace.
2.  Create a `Cargo.toml` for the example:

    ```toml
    [package]
    name = "composite_example"
    version = "0.1.0"
    edition, deps etc from the workspace toml.
    ```

3.  Create a `main.rs` in the `examples` directory:

```rust
// In /examples/main.rs
use bevy::prelude::*;
use bevy_mesh_boolean::*;
use bevy_step_loader::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            MeshBooleanPlugin,
            StepPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, perform_boolean_op_on_loaded_meshes)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Load two STEP files
    let handle1: Handle<StepAsset> = asset_server.load("model1.step");
    let handle2: Handle<StepAsset> = asset_server.load("model2.step");

    // Spawn entities with the loaded meshes
    let entity1 = commands.spawn((
        PbrBundle {
            mesh: handle1.mesh.clone(),
            material: materials.add(Color::rgb(0.9, 0.2, 0.2).into()),
            transform: Transform::from_xyz(-0.5, 0.0, 0.0),
            ..default()
        },
    )).id();
    
    let entity2 = commands.spawn((
        PbrBundle {
            mesh: handle2.mesh.clone(),
            material: materials.add(Color::rgb(0.2, 0.2, 0.9).into()),
            transform: Transform::from_xyz(0.5, 0.0, 0.0),
            ..default()
        },
    )).id();
    
    // Add the BooleanMesh component to mark them for boolean operations
    commands.entity(entity1).insert(BooleanMesh { op: BooleanOp::Primary, secondary_entity: entity2 });
    commands.entity(entity2).insert(BooleanMesh { op: BooleanOp::Secondary, secondary_entity: entity1 });
}

fn perform_boolean_op_on_loaded_meshes(
    mut op_state: ResMut<BooleanOpState>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    // A simple system to trigger a boolean operation
    if keyboard_input.just_pressed(KeyCode::Space) {
        *op_state = BooleanOpState::Subtract;
    }
}
```
