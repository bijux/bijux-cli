# Replay contract

## Scope
Defines replay semantics, semantic diff interpretation, provenance boundaries, and explicit non-goals.

## Replay definition
Replay in this repository means:

- execute from captured graph + runtime artifacts
- compare semantic outputs and node outcomes against prior run evidence
- emit explicit reason report for equivalence or mismatch

Replay is not a byte-for-byte filesystem restore mechanism.

## Authoritative inputs

Replay may consult:

- `manifest.json`
- graph snapshot and graph fingerprint
- node traces and statuses
- outputs index and output hashes
- provenance markers and replay source metadata

Replay must not consult ambient host state as authoritative evidence.

## Semantic diff mode

`dag diff` and `dag runs diff` support semantic comparison mode and emit:

- replay equivalence boolean
- mismatch reasons
- grouped mismatch causes
- replay reason summary

## Replay explain mode

`--explain` output groups mismatch causes by class:

- `manifest_drift`
- `graph_semantics`
- `node_outcomes`
- `artifact_payload`

## Fixture families

Replay fixture family includes:

- `evidence/cache/replay/match_case.json`
- `evidence/cache/replay/mismatch_case.json`
- `evidence/cache/replay/corruption_case.json`
- `evidence/cache/replay/unsupported_version_case.json`

## What replay cannot prove

Replay cannot prove:

- equivalence to uncaptured external side effects
- equivalence when authoritative artifacts are missing
- compatibility across unsupported historical or future formats
- equivalence of non-semantic metadata fields intentionally ignored by contract

## Related tests

- `tests/e2e/replay/*`
- `crates/bijux-dag-app/tests/e2e_integration_scenarios.rs`
- `crates/bijux-dag-app/tests/replay_contract.rs`
- `crates/bijux-dag-app/src/diff.rs` unit tests

## Related schemas

- `configs/schema/operator/replay_diff.schema.json`
- `configs/schema/run_manifest.schema.json`
- `configs/schema/node_trace.schema.json`

## Versioning and change policy
Replay semantics changes require explicit compatibility decision and updated replay fixture and schema coverage.
