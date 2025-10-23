# bevy_step_loader

A Bevy plugin that adds support for loading STEP and STP files as Bevy Mesh assets.

## Features

- Load STEP and STP files as Bevy assets
- Two triangulation backends: Foxtrot (Pure Rust) and OpenCascade (OCCT) 
- Configurable via feature flags
- Automatic camera positioning for optimal viewing
- Mesh statistics reporting (vertex/triangle counts)
- Real-time status updates through UI

## Triangulation Backends

This crate supports two different triangulation engines with different trade-offs:

### Foxtrot (Default)
- **Pros**: Pure Rust implementation, faster STEP parsing and triangulation, smaller binary size
- **Cons**: Less robust triangulation, particularly with complex geometries and NURBS surfaces

### OpenCascade (OCCT) - Optional Feature
- **Pros**: More robust triangulation, better handling of complex geometries and NURBS, well-established tooling
- **Cons**: C++ wrapper dependency, slower triangulation, larger binary size

## Prerequisites

For the OpenCascade backend, you need a C++ library to link into, so install some `libstdc++`:

```sh
sudo apt update
sudo apt install libstdc++-12-dev
```

Or check your distribution's package manager for equivalent packages.

## Usage

### Basic Usage (with default Foxtrot backend)

```toml
[dependencies]
bevy_step_loader = { path = "path/to/crates/bevy_step_loader" }
```

```rust
use bevy::prelude::*;
use bevy_step_loader::StepPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, StepPlugin))
        .run();
}

// In your systems, you can load STEP files directly:
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let step_handle = asset_server.load("model.step");
    commands.spawn(bevy::pbr::PbrBundle {
        mesh: step_handle,
        ..default()
    });
}
```

### With OpenCascade backend

To use the more robust OpenCascade backend, enable the `opencascade` feature flag:

```toml
[dependencies]
bevy_step_loader = { path = "path/to/crates/bevy_step_loader", features = ["opencascade"] }
```

The usage remains the same, but the triangulation will be handled by OpenCascade instead of Foxtrot.

## Automatic Camera Positioning

The plugin includes automatic camera positioning that:
- Calculates optimal viewing distances based on model bounds
- Provides 45-degree orthographic views for technical visualization
- Automatically frames models to fill 80% of the screen on the longest axis
- Supports orbiting camera movement for 3D inspection

## Supported File Extensions

- `.step`
- `.stp`

## Examples

### load_step_file
Basic example showing how to load STEP files and display mesh statistics.

### simple_step_display
STEP file loading with automatic camera positioning and smooth orbiting view.

## License

This project is licensed under your chosen license. See the LICENSE file for details.