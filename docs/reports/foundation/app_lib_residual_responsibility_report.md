# App lib.rs Residual Responsibility Report

generated_from: `crates/bijux-dag-app/src/lib.rs`

Residual responsibilities intentionally retained in `lib.rs`:

1. CLI entry, argument parsing, and high-level command dispatch.
2. Shared utility functions used across commands and routes.
3. Legacy compatibility helpers pending future reduction.

Responsibilities removed from `lib.rs` in this extraction wave:

- capability-matrix branching for equivalence proof moved to `routes/surface_routes.rs`.
- route-level run history/timeline/tree dispatch handled by `routes/runs_routes.rs`.
