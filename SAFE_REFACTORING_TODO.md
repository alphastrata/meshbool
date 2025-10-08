# Safe Refactoring Todo List for MeshBool

## Overview
The current meshbool implementation uses unsafe raw pointers for circular doubly-linked lists in the `Vert` struct. This needs to be refactored to use safe index-based references instead.

## Current Status
- [x] Documented unsafe patterns in `polygon.rs`
- [x] Modified `Vert` struct to use indices instead of raw pointers
- [x] Updated accessor methods to use indices
- [ ] Update all call sites to pass polygon references to accessor methods
- [ ] Remove all unsafe code blocks
- [ ] Ensure all functionality works correctly after refactoring
- [ ] Verify performance is not significantly impacted

## High-Priority Items (Must Fix Before Proceeding)

### 1. Update Accessor Method Calls
The biggest issue is that all calls to the accessor methods (`left()`, `right()`, `left_mut()`, `right_mut()`) need to be updated to pass the polygon reference.

**Estimated effort**: 60+ call sites to update
**Risk**: High - affects many parts of the codebase

**Examples of calls that need updating**:
- `vert.left()` → `vert.left(&polygon)`
- `vert.right()` → `vert.right(&polygon)`
- `vert.left_mut()` → `vert.left_mut(&mut polygon)`
- `vert.right_mut()` → `vert.right_mut(&mut polygon)`

### 2. Update Field Access Patterns
Direct field access that was previously done through pointer dereferencing now needs to use indexed access.

**Examples**:
- `vert_ref.left` → Need to use `vert_ref.left_idx` with indexed access
- `vert_ref.right` → Need to use `vert_ref.right_idx` with indexed access

### 3. Remove Unused Parameters
Several functions still have unused `polygon_range` parameters that need to be removed.

## Medium-Priority Items

### 4. Update Utility Functions
Functions like `ptr2index`, `index2ptr`, and related utilities need to be updated or removed.

### 5. Fix Remaining Compilation Errors
Address all the compilation errors that appeared after the initial refactoring.

### 6. Update Tests
Ensure all existing tests still pass after the refactoring.

## Low-Priority Items

### 7. Performance Profiling
Verify that the refactored code doesn't have significant performance degradation.

### 8. Add Safety Documentation
Add comprehensive safety documentation for the refactored code.

### 9. Code Cleanup
Remove any leftover artifacts from the refactoring process.

## Detailed Task Breakdown

### Phase 1: Fix Compilation Errors (High Priority)
1. [ ] Update all calls to `left()`, `right()`, `left_mut()`, `right_mut()` methods to pass polygon references
2. [ ] Fix field access patterns that were previously direct pointer dereferences
3. [ ] Remove unused `polygon_range` parameters from functions
4. [ ] Update `ptr2index` and `index2ptr` utility functions or remove them
5. [ ] Fix all remaining compilation errors

### Phase 2: Verification (Medium Priority)
1. [ ] Run all existing tests to ensure functionality hasn't changed
2. [ ] Add new tests for the refactored components if needed
3. [ ] Benchmark performance to ensure no significant degradation

### Phase 3: Documentation and Cleanup (Low Priority)
1. [ ] Update documentation to reflect the safe refactoring
2. [ ] Remove any leftover unsafe code comments that are no longer relevant
3. [ ] Clean up any temporary variables or debugging code

## Risk Assessment

### High Risk
- Changing the fundamental data structure from pointers to indices affects the entire codebase
- Incorrect updates could break the circular list invariants
- Performance could be significantly impacted if not done carefully

### Mitigation Strategies
- Make small, incremental changes and test after each change
- Keep backups of working versions
- Use version control to track changes
- Run tests after each modification
- Profile performance to detect regressions early

## Timeline Estimate

### Phase 1: Fix Compilation Errors
- Estimated time: 4-6 hours
- Depends on the number of call sites that need updating

### Phase 2: Verification
- Estimated time: 2-3 hours
- Includes running tests and performance profiling

### Phase 3: Documentation and Cleanup
- Estimated time: 1-2 hours
- Minor cleanup tasks

### Total Estimated Time: 7-11 hours

## Next Steps

1. Create a backup of the current working version
2. Start with a small section of the codebase to test the approach
3. Gradually expand the refactoring to cover more areas
4. Run tests frequently to catch regressions early
5. Document progress and any issues encountered