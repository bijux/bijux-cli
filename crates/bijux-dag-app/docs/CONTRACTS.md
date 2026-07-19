# bijux-dag-app Contracts

Responsibility: Application orchestration services, command response modeling, and user-facing render flows.

## Scope
`bijux-dag-app` owns application orchestration for user commands. It translates validated CLI models into calls to core/runtime/artifacts APIs and formats typed responses.

## Authority
This crate is the owner of command orchestration behavior, command-level response models, and command output formatting contracts.

## Invariants
- Business orchestration is implemented here, not in `bijux-dag-cli`.
- Each command path returns a typed response model before rendering.
- This crate does not own DAG schema/model semantics (`bijux-dag-core`) or runtime execution internals (`bijux-dag-runtime`).
- `../src/lib.rs` is the only root Rust file; command logic must live in bounded domain folders.

## Internal boundaries
- `../src/commands/`: orchestration command surfaces and runtime config resolution.
- `../src/graph/`: graph-oriented command surfaces and validation entrypoints.
- `../src/read/` and `../src/write/`: input/output IO shaping boundaries.
- `../src/inspect/`: run status, doctor, and run-view presentation boundaries.
- `../src/replay/`: replay and diff command boundaries.
- `../src/cache/`, `../src/explain/`, `../src/format/`, `../src/migrate/`:
  focused domain command helpers.

## Allowed changes
- Add or evolve orchestration modules while keeping command contracts backward-compatible with `docs/bijux-dag/interfaces/compatibility-commitments.md`.
- Add formatting variants that do not weaken machine-readable contracts.

## Related tests
- `crates/bijux-dag-app/tests/output_contract.rs`
- `crates/bijux-dag-app/tests/cli_contract.rs`
- `crates/bijux-dag-app/tests/e2e_integration_scenarios.rs`

## Related schemas
- `configs/dag/schema/run_manifest.schema.json`
- `configs/dag/schema/node_trace.schema.json`
- `configs/dag/schema/outputs_index.schema.json`

## Versioning and change policy
Contract-preserving changes are additive. Breaking changes require explicit CLI compatibility review and docs updates in the same change.
