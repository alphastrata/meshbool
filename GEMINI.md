# GEMINI.md

## Project Overview

This project, `meshbool`, is a Rust library for 3D mesh manipulation. It provides functionalities for creating, transforming, and performing boolean operations on 3D meshes. The library is designed to be a re-implementation of the `manifold-rs` library, and it includes a comprehensive test suite to compare the results of the two libraries.

The core data structure is `Impl`, which represents a 3D manifold. The library provides functions for creating basic shapes like cubes, and for performing transformations like translation, rotation, and scaling. The main feature of the library is the set of boolean operations: `union`, `difference`, and `intersection`.

The project also includes functionality for cross-sections, convex hulls, signed distance fields (SDF), and mesh smoothing, although some of these are still under development.

## Building and Running

This is a Rust library, so it can be built and tested using Cargo.

*   **Build:** `cargo build`
*   **Run tests:** `cargo test`

## Development Conventions

The code follows standard Rust conventions. The `tests` directory contains a suite of integration tests that compare the output of `meshbool` with `manifold-rs`. This indicates a strong emphasis on correctness and compatibility with the original library.

The `ALGORITHM_IMPLEMENTATION_PLAN.md` file suggests a structured approach to development, with a clear plan for implementing the core algorithms.

The code is well-documented, with comments explaining the purpose of each function and data structure.
