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
