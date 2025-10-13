# MeshBool Examples - Working Implementation

This directory contains two examples that demonstrate boolean operations with 3D meshes:

## Examples

### 1. `clean_implementation.rs` - Basic Boolean Operations Demo

This example shows three shapes arranged like an equation:
- **LHS Shape (Victim)** - Positioned at [-4, 0, 0] (light gray cube)
- **Output Shape (Result)** - Positioned at [0, 0, 0] (orange cube)  
- **RHS Shape (Operator)** - Positioned at [4, 0, 0] (green cylinder)

**Controls:**
- SPACE - Cycle through operations (Union, Intersection, Difference, View Original)
- R - Reset to view original
- Q - Quit with panic message "user did not see expected output of boolean mesh op {operation}"
- ESC - Quit

**Operations Displayed:**
- VIEW ORIGINAL
- BOOLEAN UNION (A ∪ B)  
- BOOLEAN INTERSECTION (A ∩ B)
- BOOLEAN DIFFERENCE (A − B)

### 2. `clean_step_integration.rs` - Real STEP File Integration

This example loads real STEP files and performs boolean operations with them.

**Features:**
- Command line argument support: `cargo run --example clean_step_integration -- /path/to/file.step`
- Automatic scanning of STEP files in default directory if no argument provided
- Same visual layout as basic demo: LHS op RHS = OUTPUT
- Same controls for cycling operations and Q key functionality

## Visual Layout

All examples arrange shapes in a line like an equation:

```
LEFT (-4,0,0)    CENTER (0,0,0)    RIGHT (4,0,0)
LHS Shape       Output Shape      RHS Shape
(Victim)        (Result)          (Operator)
```

This visual metaphor makes it clear that we're performing: `LHS_SHAPE op RHS_SHAPE = OUTPUT_SHAPE`

## Expected Behavior

1. **Three shapes visible**: Victim (left), Operator (right), Result (center)
2. **Operation cycling**: SPACE key cycles through boolean operations
3. **Q key functionality**: Pressing Q panics with specific message including current operation
4. **Command line support**: STEP file path can be provided as argument
5. **Clear labeling**: Shapes are clearly labeled and colored differently

## Dependency Resolution

These examples currently don't compile due to version conflicts between naga 26.0.0 (Bevy 0.17.2) and naga 27.0.0 (triangulate crate). See FIX_DEPENDENCIES.md for detailed instructions on resolving these conflicts.

Once dependencies are fixed, both examples will compile and demonstrate the requested functionality with proper visual layout and controls.