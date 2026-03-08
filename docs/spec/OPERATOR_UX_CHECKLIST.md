# Operator UX Checklist

This checklist defines the minimum operator-facing quality bar for `bijux-dag-app` command surfaces.

## Scope

- Validate output remains concise and remediation-oriented.
- Plan output explains inclusion/exclusion decisions deterministically.
- Run output always reports the created run directory.
- Inspect and history output remain stable for automation and humans.
- Replay and diff output communicate equivalence status clearly.
- Prove and verify output expose integrity/completeness state directly.
- Artifact inspect output includes identity, provenance, and lineage fields.

## Contract Links

- App service boundary report: `docs/reports/foundation/app_service_boundary_report.md`
- Operator UX contract: `docs/spec/OPERATOR_UX_CONTRACT.md`
- Output concision contract: `docs/spec/OUTPUT_CONCISION_CONTRACT.md`

## Test Coverage Links

- Human snapshots: `crates/bijux-dag-app/tests/operator_human_snapshot_contracts.rs`
- Schema lockstep checks: `crates/bijux-dag-app/tests/operator_schema_lockstep_contracts.rs`
- No-panic malformed input checks: `crates/bijux-dag-app/tests/operator_input_no_panic_contracts.rs`
