# bijux-dag-core contract

Responsibility: DAG schema, parsing, canonicalization, validation, and deterministic semantic graph logic.

## Responsibility
`bijux-dag-core` owns DAG model, parse, validation, resolve, canonicalization, topology, and fingerprint semantics.

## Internal boundaries
- `src/lib.rs` is a thin export surface and should not contain core algorithms.
- `graph/model.rs` owns graph domain types.
- `graph/canonical.rs` owns canonicalization and normalization.
- `graph/topology.rs` owns deterministic ordering.
- `pipeline/*` owns parse, resolve, and validate entrypoints.
- `analysis/*` owns fingerprinting and semantic analysis.
- `planner/*` owns lowering and planning surfaces.
- `build/*` owns authoring helpers and compile-oriented wrappers around the kernel.
- `contracts/*` owns error and compatibility contract types.

## Purity boundary
Core is pure logic and data transformation.

Forbidden direct dependencies in core source:
- filesystem APIs
- process execution APIs
- environment-variable reads
- wall-clock/time sourcing

Allowed utility dependencies:
- serialization
- hashing
- collections and deterministic ordering utilities

## Validation model
Validation diagnostics must carry stable IDs and severities and remain documented in `docs/spec/VALIDATION_RULES.md`.

## Architectural guardrails
- Domain types should stay independent from compile-orchestration conveniences.
- New algorithms belong in focused modules, not in `src/lib.rs`.
- Integration-oriented wrappers must not become the primary place where core semantics live.
