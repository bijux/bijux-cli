# Kernel API Surface Report

Date: 2026-03-07

## Kernel crates

- `bijux-dag-core`
- `bijux-dag-runtime` (kernel-owned submodules only, per `KERNEL_BOUNDARY_CONTRACT.md`)

## Stable kernel surface categories

- canonical graph identity and parsing
- planner and execution-plan contracts
- deterministic scheduler/state-machine contracts
- artifact identity, run manifest, cache/replay semantics

## Excluded from kernel API surface

- CLI routing/rendering
- dev governance and evidence report orchestration
- modeled/simulated distributed platform declarations
- AI/operator-assist and control-plane policy narration

## CI enforcement

- `crates/bijux-dev-dag/tests/kernel_boundary_contracts.rs`
- `crates/bijux-dev-dag/tests/no_cli_in_runtime.rs`
- `crates/bijux-dev-dag/tests/no_runtime_in_core.rs`
