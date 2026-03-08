# Core crate tests

This directory contains structural and compatibility tests for the core kernel.

- `compat.rs` validates canonical and fingerprint snapshots against contract fixtures.
- `graph_kernel_determinism.rs` covers canonicalization, fingerprint, and resolution determinism without relying on crate-root unit tests.
- Fixtures for compatibility checks live in `crates/bijux-dag-core/tests/compat/v0.1/`.
