# Kernel Public API Audit

Date: 2026-03-07

## Scope audited

- `crates/bijux-dag-core/src/lib.rs`
- `crates/bijux-dag-runtime/src/lib.rs`
- runtime kernel module tree: `crates/bijux-dag-runtime/src/runtime_core/**`

## Findings

1. Core and runtime still expose broad historical public surfaces from crate root.
2. Kernel-safe categories are now explicitly documented in `KERNEL_BOUNDARY_CONTRACT.md`.
3. Runtime modeled/platform exports remain outside kernel authority and are tracked for future demotion.

## Kernel API target

- Graph canonicalization and identity
- Planner contracts and execution-plan identity
- Scheduler/readiness/state transitions
- Artifact lineage and replay/diff verification

## Action rules

- New kernel-facing additions must satisfy kernel dependency policy and kernel hygiene tests.
- Non-kernel exports must not be used as release-proof claims.
