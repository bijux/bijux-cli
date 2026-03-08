# ADR: Dev DAG Long-Term Role

## Status

Accepted

## Context

`bijux-dev-dag` grew into a broad command surface spanning release-critical, maintenance, advisory, and compatibility functions. Without explicit role boundaries, the surface risks bloat and operator confusion.

## Decision

1. Maintain explicit purpose classification for dev-dag commands.
2. Keep compact release-critical and maintenance command packs as primary entrypoints.
3. Treat advisory and legacy commands as non-primary surfaces.
4. Require explicit signal ownership for new dev-dag commands.

## Consequences

- Release-critical governance remains machine-stable and focused.
- Maintenance workflows stay available without polluting primary operator narratives.
- Surface growth is constrained by suite- and contract-enforced ownership boundaries.

## Enforcement

- `configs/suites/dev_dag_contraction_verification.json`
- `crates/bijux-dev-dag/tests/dev_dag_surface_guarantees_contracts.rs`
