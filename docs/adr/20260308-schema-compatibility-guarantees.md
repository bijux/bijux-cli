# ADR: Schema Compatibility Guarantees

## Status

Accepted

## Context

Execution portability and operator trust depend on deterministic schema compatibility rules that are enforced by tests and CI controls, not by convention.

## Decision

We guarantee the following:

1. Stable schema files are hash-frozen and drift is merge-blocking until compatibility review is explicit.
2. Supported historical versions remain accepted for declared compatibility windows.
3. Unsupported versions are rejected with classified failures.
4. Compatibility fixtures are mandatory for graph, run, artifact, proof, diff, and explain surfaces.
5. Schema policy documentation and changelog artifacts are mandatory governance outputs.

## Consequences

- Schema changes require explicit migration and changelog work.
- CI failures surface compatibility drift early.
- Operators receive durable compatibility diagnostics through generated reports and dashboards.

## Enforcement

- `crates/bijux-dev-dag/tests/schema_governance_contracts.rs`
- `crates/bijux-dev-dag/tests/schema_evolution_completion_contracts.rs`
- `crates/bijux-dev-dag/tests/proof_schema_compatibility_contracts.rs`
- `crates/bijux-dev-dag/tests/schema_compatibility_guarantees_contracts.rs`
