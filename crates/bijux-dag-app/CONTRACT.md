# bijux-dag-app Contract

## Scope
`bijux-dag-app` owns application orchestration for user commands. It translates validated CLI models into calls to core/runtime/artifacts APIs and formats typed responses.

## Authority
This crate is the owner of command orchestration behavior, command-level response models, and command output formatting contracts.

## Invariants
- Business orchestration is implemented here, not in `bijux-dag-cli`.
- Each command path returns a typed response model before rendering.
- This crate does not own DAG schema/model semantics (`bijux-dag-core`) or runtime execution internals (`bijux-dag-runtime`).

## Allowed changes
- Add or evolve orchestration modules while keeping command contracts backward-compatible per [docs/spec/CLI_BACKWARD_COMPATIBILITY.md](/Users/bijan/bijux/bijux-dag/docs/spec/CLI_BACKWARD_COMPATIBILITY.md).
- Add formatting variants that do not weaken machine-readable contracts.

## Related tests
- `crates/bijux-dag-app/tests/output_contract.rs`
- `crates/bijux-dag-app/tests/cli_contract.rs`
- `crates/bijux-dag-app/tests/e2e_integration_scenarios.rs`

## Related schemas
- `configs/schema/run_manifest.schema.json`
- `configs/schema/node_trace.schema.json`
- `configs/schema/outputs_index.schema.json`

## Versioning and change policy
Contract-preserving changes are additive. Breaking changes require explicit CLI compatibility review and docs updates in the same change.
