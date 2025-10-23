# Bevy Plugins Workspace

This workspace contains two Bevy plugins that provide mesh boolean operations and STEP file loading capabilities.

## Plugins

### bevy_mesh_boolean
A plugin that provides capabilities for performing boolean operations (intersection, union, subtraction) on Bevy meshes using the manifold-rs library for robust geometric computations.

### bevy_step_loader
A plugin that introduces a custom asset loader for `.step` and `.stp` files, enabling Bevy to load them as assets with automatic triangulation using the Foxtrot STEP parser.

## Project Structure

```
/bevy_plugins_workspace
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
|--/examples
|  |-- main.rs (composite example)
|  |-- step_subtract_demo.rs (focused STEP subtract demo)
|  |-- simple_step_display.rs (basic STEP display with auto-camera)
|-- Cargo.toml (workspace root)
|-- rust-toolchain.toml
```

## Requirements

- Rust 1.88 or higher (managed automatically via rust-toolchain.toml)
- Git (for fetching dependencies)

//TODO: bevy version guide, we start with compat at 0.17.0
//TODO: minimum rustc version

## Getting Started

### Prerequisites

Make sure you have Rust 1.88+ installed. The project includes a `rust-toolchain.toml` file that will automatically use the correct version when you work with this project.

### Building and Running

1. **Clone or download the repository**

2. **To run the boolean operations example:**
   ```bash
   cd crates/bevy_mesh_boolean
   cargo run --example boolean_op_demo
   ```

3. **To run the STEP loader example:**
   ```bash
   cd crates/bevy_step_loader
   cargo run --example load_step_file
   ```

4. **To run the composite example (STEP file with boolean operation):**
   ```bash
   cd examples
   cargo run
   ```

5. **To run the focused STEP subtract demo:**
   ```bash
   cd examples
   cargo run --bin step_subtract_demo
   ```

6. **To run the simple STEP display example:**
   ```bash
   cd examples
   cargo run --bin simple_step_display
   ```

## Usage

### Using bevy_mesh_boolean in your own project

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
bevy_mesh_boolean = { path = "path/to/crates/bevy_mesh_boolean" }
```

Then add the plugin to your Bevy app:

```rust
use bevy::prelude::*;
use bevy_mesh_boolean::MeshBooleanPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MeshBooleanPlugin))
        .run();
}
```

### Using bevy_step_loader in your own project

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
bevy_step_loader = { path = "path/to/crates/bevy_step_loader" }
```

Then add the plugin to your Bevy app:

```rust
use bevy::prelude::*;
use bevy_step_loader::StepPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, StepPlugin))
        .run();
}
```

### Composite Usage

Both plugins can be used together for advanced workflows:

```rust
use bevy::prelude::*;
use bevy_mesh_boolean::MeshBooleanPlugin;
use bevy_step_loader::StepPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            MeshBooleanPlugin,
            StepPlugin,
        ))
        .run();
}
```

## Features

### Automatic Camera Positioning
The STEP loader includes automatic camera positioning that calculates optimal viewing distances based on model bounds and provides 45-degree orthographic views.

### Boolean Operations Support
- **Intersection**: Creates a mesh representing the overlapping volume
- **Union**: Combines two meshes into a single unified mesh
- **Subtraction**: Cuts one mesh from another (difference operation)

### STEP File Loading
- Supports both `.step` and `.stp` file extensions
- Automatic triangulation of loaded geometry
- Mesh statistics reporting (vertex/triangle counts)
- Automatic bounding box calculation for optimal camera framing

### Real-time Visualization
- Orbiting camera view that smoothly rotates around loaded models
- Multiple lighting setups for optimal model illumination
- Real-time status updates through UI text overlays
- Automatic scaling and positioning based on model dimensions

## Controls

### In the examples:

- **Space**: Cycle through boolean operations (None → Intersect → Union → Subtract → None)
- **B**: Set operation to Subtract (Cut)
- **I**: Set operation to Intersect
- **U**: Set operation to Union
- **N**: Set operation to None

## Examples

### boolean_op_demo
Demonstrates boolean operations between basic geometric shapes (cube and sphere) with real-time visualization and controls.

### load_step_file
Shows how to load STEP files as Bevy assets with automatic mesh statistics reporting and camera positioning.

### composite_example
Loads a STEP file and performs boolean operations on it using a cube that's half the size of the STEP model, demonstrating both plugins working together.

### step_subtract_demo
Focused demonstration of loading a STEP file, spawning a subtraction cube, and performing a boolean subtract operation with orbiting camera view.

### simple_step_display
Basic STEP file loading and display with automatic camera positioning to optimally frame the model.

## Dependencies

This project uses several external dependencies:
- `bevy` - The Bevy game engine
- `manifold-rs` - For robust boolean mesh operations
- `step` - For loading STEP files (from the Foxtrot repository)
- `triangulate` - For mesh triangulation
- `anyhow` - For error handling

## License

This project is licensed under your chosen license. See the LICENSE file for details.