# Evidence Governance and Severity Model

Status: accepted
Owner: evidence maintainers
Date: 2026-03-09

## Decision
Evidence is classified by release impact. Blocking evidence must stay stable and machine-verifiable; advisory evidence may evolve without blocking release.

## Consequences
- Evidence ownership is explicit and auditable.
- Evidence consumers must use approved access surfaces.
- Release gates bind only to blocking evidence classes.

## Merged Decision Record
This ADR is standalone. The historical decision text merged into this record is included below.

### SOURCE: 20260308-EVIDENCE-MINIMALISM.md
# ADR: Evidence Minimalism

## Status

Accepted

## Context

Evidence surfaces expanded in breadth and overlap, reducing decision clarity and increasing maintenance burden.

## Decision

1. Each evidence family must declare severity, audience, source-of-truth, and release-review relevance.
2. Duplicate or low-value evidence outputs should be merged into canonical decision surfaces.
3. Release-critical and advisory evidence paths must remain isolated in governance behavior.
4. Compact evidence index and claim mapping are required for operator and maintainer clarity.

## Consequences

- Evidence decision value is easier to evaluate.
- Governance can block low-signal evidence growth.
- Release review consumes a smaller and clearer evidence set.

## Enforcement

- `configs/policy/evidence_family_governance.json`
- `configs/suites/evidence_signal_sharpening_verification.json`
- `crates/bijux-dev-dag/tests/evidence_signal_quality_contracts.rs`

### SOURCE: 20260308-EVIDENCE-SEVERITY-RATIONALIZATION.md
# ADR: Evidence Severity Classes and Rationalization

- Date: 2026-03-08
- Status: Accepted

## Context

Evidence verification surfaces were present, but severity and audience intent were spread across multiple files. This made it hard to answer: which evidence blocks release, which supports maintainers, and which remains advisory.

## Decision

Adopt three explicit evidence severity classes with one policy source:

- `release-critical`: must execute in green release paths.
- `release-supporting`: supports governance and maintenance, not a release blocker by default.
- `advisory`: optional operator signal, never release-blocking by default.

Governed command/report metadata now requires:

- declared `severity_class`
- declared `audience`
- one mapped `docs_page`

## Consequences

- Release readiness review uses the compact release evidence pack.
- Advisory review uses a separate compact advisory pack.
- Duplicate evidence signals are consolidated under canonical owner outputs.
- New evidence commands/outputs without audience, severity, and docs mapping fail governance contracts.

## Artifacts

- `configs/policy/evidence_rationalization_policy.json`
- `docs/reports/foundation/release_critical_evidence_commands_only_report.md`
- `docs/reports/foundation/release_supporting_evidence_commands_report.md`
- `docs/reports/foundation/advisory_only_evidence_commands_report.md`
- `docs/reports/foundation/evidence_outputs_duplicate_signal_report.md`
- `docs/reports/foundation/EVIDENCE_DOCS_MAPPING_REPORT.md`
- `docs/reports/foundation/EVIDENCE_SUITE_EXERCISE_MAPPING_REPORT.md`
- `docs/reports/foundation/evidence_commands_not_exercised_in_ci_report.md`
- `docs/reports/foundation/compact_release_evidence_pack.json`
- `docs/reports/foundation/compact_advisory_evidence_pack.json`

### SOURCE: 20260307-EVIDENCE-CONSUMER-ACCESS-ARCHITECTURE.md
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
