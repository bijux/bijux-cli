# Replay Contract

## Scope
Defines replay guarantees, comparison semantics, and non-goals.

## Guarantees
- Replay compares against captured run artifacts.
- Replay diagnostics identify semantic differences.

## Non-goals
- Replay does not promise equivalence to ambient host state not captured in artifacts.

## Related tests
- `tests/e2e/replay/*`
- `crates/bijux-dag-app/tests/e2e_integration_scenarios.rs`

## Related schemas
- `configs/schema/run-manifest.schema.json`
- `configs/schema/node-trace.schema.json`

## Versioning and change policy
Replay semantics changes require explicit compatibility decision and updated e2e coverage.
