---
title: Run Directory Ownership
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Run Directory Ownership

Run-directory ownership is split so storage authority, runtime orchestration,
and operator verification remain distinct.

## Ownership table

- `bijux-dag-artifacts`: run-dir finalization, verification, and marker ownership
- `bijux-dag-runtime`: runtime production of governed artifact content
- `bijux-dag-app`: import, export, and operator-facing verification routes
- `bijux-dev`: maintainer governance checks for run-dir hardening

## Related tests

- `crates/bijux-dag-app/tests/run_dir_import_export_contract.rs`
- `crates/bijux-dag-artifacts/tests/artifact_hardening_contracts.rs`
- `crates/bijux-dev/tests/run_dir_import_export_hardening_contracts.rs`
