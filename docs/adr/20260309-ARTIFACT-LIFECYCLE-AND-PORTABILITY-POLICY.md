# Artifact Lifecycle and Portability Policy

Status: accepted
Owner: artifact maintainers
Date: 2026-03-09

## Decision
Artifact identity, lifecycle, and bundle portability are first-class guarantees with strict integrity verification.

## Consequences
- Export/import behavior is contract-governed.
- Artifact durability and lineage stay evidence-backed.

## Merged Decision Record
This ADR is standalone. The historical decision text merged into this record is included below.

### SOURCE: 20260308-ARTIFACT-LIFECYCLE-GUARANTEES.md
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
  - `docs/reports/foundation/ARTIFACT_LIFECYCLE_241_260_STATUS_REPORT.md`
- Integrity and invariant reports:
  - `docs/reports/foundation/ARTIFACT_STORE_INTEGRITY_REPORT.md`
  - `docs/reports/foundation/ARTIFACT_LIFECYCLE_INVARIANTS_REPORT.md`
- Lifecycle dashboard:
  - `docs/reports/foundation/ARTIFACT_LIFECYCLE_DASHBOARD.md`
- Verification suite:
  - `configs/suites/artifact_lifecycle_invariants.json`

## Consequences

- Artifact lifecycle is treated as a governed correctness surface.
- Future lifecycle changes must preserve mapped invariants and contract coverage.

### SOURCE: 20260308-BUNDLE-PORTABILITY-GUARANTEES.md
# ADR: Bundle Portability Guarantees

- Status: accepted
- Date: 2026-03-08

## Context

Bundle import/export is a core portability boundary across operators, environments,
and replay workflows. Portability correctness requires stable format behavior,
explicit failure diagnostics, and compatibility governance.

## Decision

Bundle portability guarantees are:

1. Export/import behavior remains reproducible and verify-only safe.
2. Corrupted, truncated, or unsupported bundles are rejected with clear diagnostics.
3. Imported bundles remain replay-compatible and provenance-preserving.
4. Schema drift visibility and fsck verification remain enforced in governance suites.

## Enforcement

- Status mapping:
  - `docs/reports/foundation/BUNDLE_PORTABILITY_261_280_STATUS_REPORT.md`
- Portability and diagnostics reports:
  - `docs/reports/foundation/BUNDLE_PORTABILITY_REPORT.md`
  - `docs/reports/foundation/BUNDLE_IMPORT_DIAGNOSTICS_REPORT.md`
- Compatibility dashboard:
  - `docs/reports/foundation/BUNDLE_COMPATIBILITY_DASHBOARD.md`
- Verification suite:
  - `configs/suites/bundle_portability_verification.json`

## Consequences

- Bundle portability becomes an explicit correctness and operator-trust contract.
- Changes to bundle format or import behavior must preserve governed compatibility signals.
