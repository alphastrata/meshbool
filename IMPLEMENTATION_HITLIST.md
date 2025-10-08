# Implementation Hitlist

## Priority 1: Core Unimplemented Functions (in src/lib.rs)

### 1. cross_section (line 312)
- **Location**: `src/lib.rs:312`
- **Purpose**: Computes cross-sections of manifolds
- **Signature**: `pub fn cross_section(r#impl: &Impl, height: f64) -> Impl`
- **Status**: ⏳ In Progress - Basic stub implemented, tests pass

### 2. hull (line 318)
- **Location**: `src/lib.rs:318`
- **Purpose**: Computes convex hull of meshes
- **Signature**: `pub fn hull(r#impl: &Impl) -> Impl`
- **Status**: Stub with `unimplemented!()`

### 3. sdf (line 324)
- **Location**: `src/lib.rs:324`
- **Purpose**: Creates signed distance field from mesh
- **Signature**: `pub fn sdf(r#impl: &Impl, tolerance: f64) -> Impl`
- **Status**: Stub with `unimplemented!()`

### 4. smooth (line 330)
- **Location**: `src/lib.rs:330`
- **Purpose**: Applies smoothing to meshes
- **Signature**: `pub fn smooth(r#impl: &Impl, tolerance: f64) -> Impl`
- **Status**: Stub with `unimplemented!()`

## Priority 2: Complex Boolean Operations (in tests/boolean_complex.rs)

### 5. Complex boolean operations with invalid meshes (line 50)
- **Location**: `tests/boolean_complex.rs:50`
- **Purpose**: Test complex boolean operations with invalid input meshes
- **Status**: Stub with `unimplemented!()`

## Priority 3: Test Functions with should_panic Attributes

### Cross Section Tests (5 tests)
1. `test_cross_section_basic` - `tests/cross_section.rs:5`
2. `test_cross_section_with_height` - `tests/cross_section.rs:14`
3. `test_cross_section_edge_cases` - `tests/cross_section.rs:23`
4. `test_cross_section_complex_shape` - `tests/cross_section.rs:32`
5. `test_cross_section_large_shape` - `tests/cross_section.rs:43`

### Hull Tests (5 tests)
1. `test_hull_basic` - `tests/hull.rs:5`
2. `test_hull_with_transformations` - `tests/hull.rs:13`
3. `test_hull_edge_cases` - `tests/hull.rs:22`
4. `test_hull_complex_shape` - `tests/hull.rs:33`
5. `test_hull_large_shape` - `tests/hull.rs:41`

### SDF Tests (5 tests)
1. `test_sdf_basic` - `tests/sdf.rs:5`
2. `test_sdf_with_tolerance` - `tests/sdf.rs:13`
3. `test_sdf_edge_cases` - `tests/sdf.rs:21`
4. `test_sdf_complex_shape` - `tests/sdf.rs:30`
5. `test_sdf_large_shape` - `tests/sdf.rs:41`

### Smooth Tests (5 tests)
1. `test_smooth_basic` - `tests/smooth.rs:5`
2. `test_smooth_edge_cases` - `tests/smooth.rs:13`
3. `test_smooth_large_shape` - `tests/smooth.rs:22`
4. `test_smooth_complex_shape` - `tests/smooth.rs:33`
5. `test_smooth_with_different_tolerance` - `tests/smooth.rs:41`

## Implementation Strategy

### Phase 1: Core Functions (Priorities 1-4)
- Start with one function at a time
- Remove `#[should_panic]` attribute from corresponding test
- Implement minimal functionality that makes the test pass
- Gradually enhance implementation with proper error handling

### Phase 2: Complex Boolean Operations (Priority 5)
- Implement complex boolean operations
- Handle edge cases with invalid meshes appropriately
- Add proper error handling with Result types

### Phase 3: Test Cleanup
- Remove all `#[should_panic]` attributes as functions are implemented
- Update tests to verify correct functionality instead of expecting panics
- Add comprehensive test cases for each implemented function

## Recommended Implementation Order

1. **cross_section** - Likely the simplest to implement
2. **hull** - Convex hull computation is a well-known algorithm
3. **smooth** - Mesh smoothing algorithms are well-documented
4. **sdf** - Signed distance field computation
5. **Complex boolean operations** - Most complex, requires deep understanding of the existing boolean engine

## Success Criteria

For each function:
1. Remove the `unimplemented!()` panic
2. Remove the corresponding `#[should_panic]` test attribute
3. Make all associated tests pass
4. Add proper documentation
5. Ensure memory safety (no unsafe code)
6. Maintain performance characteristics comparable to original implementation