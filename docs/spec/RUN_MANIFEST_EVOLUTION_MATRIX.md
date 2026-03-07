# Run Manifest Evolution Matrix

| Version | Status | Required migration behavior |
| --- | --- | --- |
| `run-manifest/v0.1` | supported | parse and verify with strict required keys |
| pre-`v0.1` | unsupported | fail with compatibility diagnostics |

## Test matrix owner

- `crates/bijux-dev-dag/tests/run_manifest_evolution_contracts.rs`
