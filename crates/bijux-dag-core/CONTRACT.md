# bijux-dag-core contract

Responsibility: DAG schema, parsing, canonicalization, validation, and deterministic semantic graph logic.

## Responsibility
`bijux-dag-core` owns DAG model, parse, validation, resolve, canonicalization, topology, and fingerprint semantics.

## Internal boundaries
- `src/lib.rs` is a thin export surface and should not contain core algorithms.
- `src/graph/model.rs` owns graph domain types.
- `src/graph/canonical.rs` owns canonicalization and normalization.
- `src/graph/topology.rs` owns deterministic ordering.
- `src/pipeline/` owns parse, resolve, and validate entrypoints.
- `src/analysis/` owns fingerprinting and semantic analysis.
- `src/build/contract.rs` owns optional packaging metadata and default application.
- `src/planner/` owns lowering and planning surfaces.
- `src/build/` owns authoring helpers and compile-oriented wrappers around the kernel.
- `src/contracts/` owns error and compatibility contract types.

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
Validation diagnostics must carry stable IDs and severities and remain documented in `docs/bijux-dag/interfaces/data-contracts.md` and `docs/bijux-dag/interfaces/error-codes.md`.

## Related schemas

- `configs/dag/schema/dag.schema.json`
- `configs/dag/schema/extension_descriptor.schema.json`
- `configs/dag/schema/graph_canonical_diff.schema.json`
- `configs/dag/schema/graph_fingerprint_explain.schema.json`
- `configs/dag/schema/migration_report.schema.json`
- `configs/dag/schema/planner_explain.schema.json`
- `configs/dag/schema/policy_config.schema.json`

## Architectural guardrails
- Domain types should stay independent from compile-orchestration conveniences.
- New algorithms belong in focused modules, not in `src/lib.rs`.
- Integration-oriented wrappers must not become the primary place where core semantics live.
- Core compilation must work directly from `Graph`; contract wrappers are optional adapters, not the primary API.
