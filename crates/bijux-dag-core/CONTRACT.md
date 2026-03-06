# bijux-dag-core contract

## Responsibility
`bijux-dag-core` owns DAG model, parse, validation, resolve, canonicalization, topology, and fingerprint semantics.

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
