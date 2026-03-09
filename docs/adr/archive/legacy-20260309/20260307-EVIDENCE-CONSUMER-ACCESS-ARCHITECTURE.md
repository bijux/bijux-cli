# ADR 20260307: Evidence Consumer Access Architecture

## Status

Accepted

## Context

Evidence ownership moved to `evidence/`, but consumers were still free to read registry files directly from arbitrary locations. This created drift risk and inconsistent diagnostics when asset IDs were missing, duplicated, or misclassified.

## Decision

Consumers must resolve evidence assets through typed access helpers instead of ad hoc filesystem reads.

The control-plane access layer is implemented in `crates/bijux-dev-dag/src/commands/evidence_access.rs` and provides:

- resolver by asset ID
- resolver by evidence family
- resolver by trust property
- resolver by consumer
- deterministic asset ordering
- duplicate ID rejection
- direct-registry-read bypass detection

Test helpers are also exposed in `crates/bijux-dag-testkit/src/lib.rs` for shared test consumers.

## Consequences

- Evidence consumer paths are explicit and auditable.
- Missing assets now fail with stable diagnostics.
- Registry bypasses become a verification failure in `verify evidence-consumers`.
- Consumer reports are generated from a single resolver path:
  - `evidence/reports/evidence_assets_to_consumers.md`
  - `evidence/reports/evidence_consumers_to_families.md`
