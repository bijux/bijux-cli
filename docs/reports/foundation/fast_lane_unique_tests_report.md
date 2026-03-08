# Fast-Lane Unique Tests Report

generated_from: `make test` inventory comparison against `make test-all`

These tests are optimized for fast signal and run in fast lane to preserve quick
feedback while still exercising identity and command-surface correctness:

- `crates/bijux-dag-app/tests/operator_surface_contracts.rs`
- `crates/bijux-dag-core/tests/spec_behavior_contract.rs`

The machine-readable source of truth is:

- `docs/reports/foundation/fast_lane_unique_inventory.json`
