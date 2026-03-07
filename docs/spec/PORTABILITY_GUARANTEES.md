# Portability Guarantees

## Exact guarantees

- Bundle schema validation for supported versions.
- Canonical graph identity preservation when graph snapshot is present.
- Import invariant checks for required structural fields.

## Fidelity-graded guarantees

- Provenance completeness when source context is redacted.
- Artifact replay equivalence when payloads are omitted (`without-artifacts`, `provenance-only`).
- Cross-environment reproducibility with backend capability drift.

## Interpretation

Import summaries expose `fidelity.level` and `fidelity.downgrade_reasons` for machine-readable portability status.
