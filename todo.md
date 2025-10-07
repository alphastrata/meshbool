# Plan for Porting Manifold Test Suite to meshbool

## Overview
The original Manifold C++ library has a comprehensive test suite located in the `test/` directory. The goal is to port these tests to the Rust-based meshbool crate, adapting them from C++ to idiomatic Rust code.

## Current meshbool test state
- Currently has only one basic test in `src/lib.rs` that tests basic cube operations (union, difference, intersection)
- No comprehensive test suite exists yet

## Original Manifold Test Suite Structure
- `boolean_test.cpp` - Boolean operations (union, intersection, difference)
- `boolean_complex_test.cpp` - Complex boolean operations
- `cross_section_test.cpp` - Cross-section functionality
- `hull_test.cpp` - Convex hull operations
- `manifold_test.cpp` - Core manifold functionality tests
- `polygon_test.cpp` - Polygon operations
- `properties_test.cpp` - Property handling tests
- `sdf_test.cpp` - Signed distance field tests
- `smooth_test.cpp` - Smoothing operations
- `samples_test.cpp` - Sample and example functionality
- `manifold_fuzz.cpp` - Fuzz testing
- `polygon_fuzz.cpp` - Polygon fuzz testing

## Porting Plan

### Phase 1: Basic Infrastructure
1. Set up proper Rust test structure with `tests/` directory
2. Create basic test utilities and helpers for mesh creation and validation
3. Adapt the simple test currently in `src/lib.rs` to the new structure
4. Establish test fixtures and common meshes (cube, sphere, cylinder, etc.)

### Phase 2: Core Functionality Tests
1. Port `boolean_test.cpp` to Rust - test basic boolean operations (union, intersection, difference)
2. Port `manifold_test.cpp` - core manifold functionality tests
3. Port `polygon_test.cpp` - polygon operations

### Phase 3: Advanced Functionality Tests
1. Port `boolean_complex_test.cpp` - complex boolean operations
2. Port `hull_test.cpp` - convex hull operations
3. Port `properties_test.cpp` - property handling tests
4. Port `smooth_test.cpp` - smoothing operations

### Phase 4: Specialized Functionality Tests
1. Port `cross_section_test.cpp` - if cross-section functionality is implemented
2. Port `sdf_test.cpp` - if SDF functionality exists in meshbool
3. Port `samples_test.cpp` - if sample functionality exists in meshbool

### Phase 5: Fuzz and Edge Case Testing
1. Adapt concepts from `manifold_fuzz.cpp` and `polygon_fuzz.cpp` for Rust
2. Create property-based tests using proptest or similar
3. Add tests for edge cases and error conditions

## Implementation Strategy
1. Create a `tests/` directory in the meshbool project root
2. Organize tests by functionality:
   - `boolean_ops.rs` - Boolean operations
   - `core.rs` - Core manifold functionality
   - `polygons.rs` - Polygon operations
   - `properties.rs` - Property handling
   - `hull.rs` - Convex hull operations
   - `smooth.rs` - Smoothing operations
3. For each test file, convert the C++ test cases to Rust equivalents
4. Use Rust's `assert!`, `assert_eq!`, etc. macros instead of C++ GoogleTest
5. Adapt mesh creation methods to use meshbool's constructor functions
6. Add proper error handling where needed

## Test Conversion Approach
1. Identify equivalent functionality in meshbool for each test
2. Adapt C++ test assertions to Rust
3. Handle any API differences between original Manifold and meshbool
4. Update any mathematical tolerances or floating-point comparisons
5. Ensure all Rust idioms and best practices are followed

## Validation Plan
1. Ensure all ported tests pass with the current meshbool implementation
2. Check that tests validate meaningful properties (mesh validity, boolean correctness, etc.)
3. Add performance benchmarks where appropriate
4. Set up CI integration for the new tests

## Timeline
- Phase 1: 1-2 days
- Phase 2: 2-3 days
- Phase 3: 3-4 days
- Phase 4: 2-3 days
- Phase 5: 2-3 days
- Total: 10-15 days for complete test suite porting

## Additional Considerations
1. Some C++ functionality may not yet exist in meshbool - make note of functionality gaps
2. Consider adding additional tests specific to Rust safety features
3. Add tests for memory safety and panic conditions
4. Ensure WASM compatibility of tests (if needed)