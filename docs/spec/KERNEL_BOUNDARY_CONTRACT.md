# Kernel Boundary Contract

## Scope

Defines the conceptual kernel boundary for bijux-dag.

## Kernel definition

Kernel means the deterministic execution truth path:

- canonical graph parsing and validation
- execution planning
- scheduler readiness and ordering
- run-state transitions
- artifact commit and lineage identity
- replay and diff semantic verification

## Modules in kernel ownership

- `crates/bijux-dag-core/src/graph/**`
- `crates/bijux-dag-core/src/pipeline/**`
- `crates/bijux-dag-core/src/planner/**`
- `crates/bijux-dag-core/src/analysis/fingerprint.rs`
- `crates/bijux-dag-runtime/src/runtime_core/**`
- `crates/bijux-dag-runtime/src/artifacts/**`
- `crates/bijux-dag-runtime/src/cache/**`
- `crates/bijux-dag-runtime/src/replay/**`
- `crates/bijux-dag-runtime/src/policy/**`

## Modules excluded from kernel ownership

- CLI route, rendering, and dev governance surfaces.
- Runtime modeled/future platform surfaces (`internal/**`, `backend/distributed/**`, `backend/runtime/*execution*` except stable local path semantics).
- AI/operator-assist and control-plane reporting modules.
- Evidence report generation and release-report formatting modules.

## Dependency invariants

- Kernel code must not depend on CLI crates.
- Kernel code must not depend on dev governance crates.
- Kernel code must not read or format evidence report content.

## Related reports

- `docs/reports/foundation/KERNEL_API_SURFACE_REPORT.md`
- `docs/reports/foundation/PUBLIC_API_SHRINK_REPORT.md`
