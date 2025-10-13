# How to Fix Dependency Conflicts

The current build is failing due to version conflicts between different versions of the `naga` crate being pulled in:

1. **naga 26.0.0** - Pulled in by Bevy 0.17.2 components
2. **naga 27.0.0** - Pulled in by the `triangulate` crate from the foxtrot repository

## Root Cause

The dependency tree looks like this:

```
bevy_render v0.17.2
└── naga v26.0.0

triangulate v0.1.0 (foxtrot)
└── naga v27.0.0
```

These two versions are incompatible because they both depend on different versions of the `termcolor` crate, causing the `WriteColor` trait conflict.

## Solution

To fix this, you need to update the foxtrot repository to use compatible versions, or pin all dependencies to use the same version of naga.

### Option 1: Update foxtrot triangulate crate

Update the triangulate crate in the foxtrot repository to use naga 26.0.0 instead of 27.0.0.

### Option 2: Pin all naga versions

Add this to your Cargo.toml files:

```toml
# In [workspace.dependencies]
naga = "26.0.0"

# In [patch.crates-io]  
naga = { version = "=26.0.0" }
```

## What the Working Code Looks Like

Once dependencies are fixed, the examples in `examples/clean_implementation.rs` and `examples/clean_step_integration.rs` will work correctly and demonstrate:

1. **Visual Layout**: Three shapes arranged like an equation: LHS op RHS = OUTPUT
2. **Command Line Arguments**: Support for loading STEP files from command line
3. **Q Key Functionality**: Panics with "user did not see expected output of boolean mesh op {operation}"
4. **Operations Cycling**: SPACE key cycles through boolean operations

The visual arrangement:
```
[-4,0,0]     [0,0,0]      [4,0,0]  
LHS_SHAPE  OUTPUT_SHAPE  RHS_SHAPE
(Victim)   (Result)     (Operator)
```

This provides a clear visual representation of boolean operations as equations.