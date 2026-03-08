# ADR: Artifact Lifecycle Guarantees

- Status: accepted
- Date: 2026-03-08

## Context

Artifact lifecycle behavior is a system-trust boundary. Operators require strong
guarantees for write atomicity, corruption recovery, lineage correctness, and GC safety.

## Decision

Artifact lifecycle guarantees are:

1. Artifact writes remain atomic and interruption-resilient.
2. Integrity verification must classify and surface checksum and metadata anomalies.
3. Lineage reconstruction and GC planning remain deterministic and explainable.
4. Lifecycle stress, durability signals, and recovery surfaces remain contract-verified.

## Enforcement

- Status mapping:
  - `docs/reports/foundation/artifact_lifecycle_241_260_status_report.md`
- Integrity and invariant reports:
  - `docs/reports/foundation/artifact_store_integrity_report.md`
  - `docs/reports/foundation/artifact_lifecycle_invariants_report.md`
- Lifecycle dashboard:
  - `docs/reports/foundation/artifact_lifecycle_dashboard.md`
- Verification suite:
  - `configs/suites/artifact_lifecycle_invariants.json`

## Consequences

- Artifact lifecycle is treated as a governed correctness surface.
- Future lifecycle changes must preserve mapped invariants and contract coverage.
