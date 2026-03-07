# bijux-dag-core contract

## Responsibility
`bijux-dag-core` owns DAG model, parse, validation, resolve, canonicalization, topology, and fingerprint semantics.

## Internal boundaries
- `src/lib.rs` is the only root Rust file.
- `build/*`: builder and compile surfaces.
- `graph/*`: graph model and topology components.
- `pipeline/*`: parse, resolve, and validate pipeline surfaces.
- `analysis/*`: effects, fingerprint, and semantic analysis.
- `planner/*`: lowering and planning surfaces.
- `contracts/*`: error and compatibility contract types.

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
