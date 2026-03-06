# Test topology and migration policy

## Decisions

- Root-level compatibility fixtures are moved to crate scope under `crates/bijux-dag-core/tests/compat/v0.1`.
- Root-level architecture checks are moved to `crates/bijux-dev-dag/tests/` to keep workspace policies in the control-plane crate.
- Root-level `tests/arch` and `tests/compat` directories are removed to avoid duplicate fixture/check ownership.

## Current layout

- `crates/bijux-dag-core/tests/compat/v0.1`: compat fixtures and parser contract checks.
- `crates/bijux-dag-core/tests/compat.rs`: crate-local fixture-driven compat assertions.
- `crates/bijux-dev-dag/tests/`: repository architecture tests.
- `crates/bijux-dag-runtime/tests/`: runtime contract and behavior tests.
- `docs/TEST_TAXONOMY.md`: repo-wide test-classification index.
