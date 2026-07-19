---
title: Run Directory Storage Contract
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Run Directory Storage Contract

Run-directory storage is a governed artifact surface, not an incidental
filesystem side effect.

## Scope

This contract covers run-dir hardening in
`crates/bijux-dag-artifacts/src/storage/hardening.rs` together with the
operator verification surface and artifact conformance tests.

## Storage guarantees

- `write_json_atomic_durable` writes governed JSON through a durable temp-and-rename flow
- `write_incomplete_run_marker` records incomplete finalization explicitly
- `finalize_run_manifest_with_mode` distinguishes complete and incomplete run finalization
- `verify_run_dir` treats missing manifest, outputs index, trace directory, and
  strict-finalization artifacts as auditable anomalies

## Run-dir finalization markers

- `manifest.json` is the active run summary
- `manifest.finalized.json` is the finalized manifest copy
- `.run-complete.json` marks a complete finalized run
- `.run-incomplete.json` marks an incomplete or interrupted finalized run

## Governed retained paths

The storage-owned finalized run-directory format centers on:

- `manifest.json`
- `graph.snapshot.json`
- `outputs/index.json`
- `nodes/<node_id>/trace.json`
- `nodes/<node_id>/attempts.json`
- `nodes/<node_id>/resolved_params.json`
- `nodes/<node_id>/inputs/index.json`
- `nodes/<node_id>/outputs/index.json`
- `observability.events.json`
- `observability.timeline.json`
- `run.log.jsonl`
- `run.schema.json`

Additional files such as `run.snapshot.json`, `run-log.index.json`,
`scheduler.checkpoint.json`, `failure-propagation.json`, and `plan.json` may be
retained for inspection or repair, but they do not replace the authoritative
manifest, trace, index, and event surfaces.

## Related tests

- `crates/bijux-dag-artifacts/tests/artifact_hardening_contracts.rs`
- `crates/bijux-dag-app/tests/run_dir_import_export_contract.rs`
- `crates/bijux-dev/tests/run_dir_import_export_hardening_contracts.rs`

## Versioning and change policy

Any incompatible change to run-dir finalization markers, strict verification
requirements, or durable storage-write guarantees must update this contract and
the linked tests in the same change.
