# Import Export Contract

## Scope
Defines export bundle formats, metadata-only behavior, and compatibility expectations.

## Invariants
- File-including export and metadata-only export have explicit, documented semantic differences.
- Import validates bundle structure before accepting artifacts.

## Related tests
- `tests/e2e/import_export/*`
- `crates/bijux-dag-app/tests/e2e_integration_scenarios.rs`

## Related schemas
- `configs/schema/run-manifest.schema.json`
- `configs/schema/outputs-index.schema.json`

## Versioning and change policy
Format changes require compatibility fixtures for supported windows.
