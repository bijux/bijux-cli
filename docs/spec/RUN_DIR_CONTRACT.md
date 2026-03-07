# Run Directory Contract

## Scope
Defines run directory layout, mandatory files, optional files, and compatibility rules.

## Required entries (authoritative)
- `manifest.json`
- `graph.snapshot.json`
- `nodes/<node_id>/trace.json`
- `outputs/index.json`

## Optional entries
- `latest` symlink
- `provenance.json`
- cache proof payloads attached to node traces

## Derived artifacts (non-authoritative)
- timeline and inspect reports reconstructed from authoritative artifacts
- analytics summaries
- comparison reports

## Verification behavior
- `dag verify` (standard): requires `manifest.json` and `outputs/index.json`.
- `dag verify --deep`: adds schema parsing checks.
- `dag verify --strict`: requires all authoritative entries and supported `manifest_version`.
- Missing optional entries must not fail standard verification.

## Ownership
- File ownership mapping is defined in `docs/spec/RUN_DIR_OWNERSHIP.md`.

## Invariants
- Paths are relative, normalized, and non-escaping.
- Historical runs are immutable after finalization.
- `latest` link updates must not mutate historical run payloads.

## Related tests
- `crates/bijux-dag-app/tests/e2e_integration_scenarios.rs`
- `crates/bijux-dag-app/tests/fault_resilience_integration.rs`
- `crates/bijux-dag-app/tests/run_dir_import_export_contract.rs`
- `crates/bijux-dag-artifacts/tests/artifact_hardening_contracts.rs`

## Related schemas
- `configs/schema/run_manifest.schema.json`
- `configs/schema/node_trace.schema.json`
- `configs/schema/outputs_index.schema.json`
- `configs/schema/operator/run_verify_report.schema.json`

## Versioning and change policy
Additive optional files are allowed. Required structure changes require compatibility review and fixture migration.
