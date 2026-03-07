# Run Directory Contract

## Scope
Defines run directory layout, mandatory files, optional files, and compatibility rules.

## Required entries
- `run-manifest.json`
- `trace/` with per-node trace files
- `outputs-index.json`

## Optional entries
- `cache-proof.json`
- exported bundle metadata

## Invariants
- Paths are relative, normalized, and non-escaping.
- Historical runs are immutable after finalization.
- `latest` link updates must not mutate historical run payloads.

## Related tests
- `crates/bijux-dag-app/tests/e2e_integration_scenarios.rs`
- `crates/bijux-dag-app/tests/fault_resilience_integration.rs`

## Related schemas
- `configs/schema/run-manifest.schema.json`
- `configs/schema/node-trace.schema.json`
- `configs/schema/outputs-index.schema.json`

## Versioning and change policy
Additive optional files are allowed. Required structure changes require compatibility review and fixture migration.
