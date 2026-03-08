# CLI Command Stability Documentation

## Stable Command Families

- graph authoring and validation: `validate`, `hash graph`, `fingerprint`
- execution and replay: `run`, `replay`, `diff`, `why-rerun`, `why-cache-missed`
- run introspection: `runs list/show/inspect/history/timeline/tree/id-explain/explain-failure`
- artifact introspection: `artifact-inspect`, `trace-artifact`
- verification and portability: `prove`, `verify`, `export`, `import`, `fsck`

## Contract Anchors

- CLI surface tests: `crates/bijux-dag-cli/tests`
- JSON schema lockstep tests: `crates/bijux-dev-dag/tests/json_output_governance_contracts.rs`
- Human-output snapshots: `crates/bijux-dag-app/tests/snapshots`
- Error code policy: `configs/policy/error_codes.json`

## Compatibility Intent

- Additive flags are preferred over behavioral redefinition.
- Existing flags are not repurposed.
- Stable commands preserve machine-readable JSON contracts across patch releases.
