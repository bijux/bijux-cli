# Import Export Contract

## Scope
Defines export bundle formats, metadata-only behavior, and compatibility expectations.

## Invariants
- File-including export and metadata-only export have explicit, documented semantic differences.
- Import validates bundle structure before accepting artifacts.
- Import validates bundle version before accepting artifacts.

## Export modes
- `dag export --manifest-only`: exports manifest/snapshot/traces/output indexes without payload files.
- `dag export --with-files`: exports bundle including output file payloads.
- `dag export --manifest-only` and `dag export --with-files` are mutually exclusive.

## Bundle shape
- Required fields: `bundle_version`, `export_mode`, `manifest`, `graph_snapshot`, `node_traces`, `outputs`.
- `export_mode=manifest-only` requires `files` to be absent or `null`.
- `export_mode=with-files` requires `files` map payload.
- `provenance.source` identifies source class (`native-run` today).

## Bundle versioning
- Current supported bundle version: `export-bundle/v0.1`.
- Unsupported versions must fail with explicit remediation.

## Related tests
- `crates/bijux-dag-app/tests/e2e_integration_scenarios.rs`
- `crates/bijux-dag-app/tests/run_dir_import_export_contract.rs`
- `crates/bijux-dag-app/tests/version_fixture_contracts.rs`
- `evidence/compat/export_bundle/v0_1_supported/bundle.json`
- `evidence/compat/export_bundle/unsupported_past/bundle.json`

## Related schemas
- `configs/schema/run_manifest.schema.json`
- `configs/schema/outputs_index.schema.json`
- `configs/schema/operator/run_verify_report.schema.json`

## Versioning and change policy
Format changes require compatibility fixtures for supported windows.
