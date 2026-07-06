---
title: Import Export Contract
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Import Export Contract

Run import and export must preserve structural evidence compatibility without
pretending that every bundle is fully materialized.

## Scope

This contract covers exported run bundles, import verification behavior, export
mode semantics, and provenance compatibility across supported origin classes.

## Bundle versioning

Supported export bundles use `export-bundle/v0.1`.

Bundles from unsupported past versions must fail import with a clear compatibility error.

## Export modes

- `dag export --manifest-only` emits manifest, graph snapshot, and structural provenance without file payloads
- `dag export --with-files` emits file payloads for importable structural replay and inspection
- `--without-artifacts` keeps outputs and files absent while preserving manifest-level structure

## Provenance contract

- `provenance.source` records the origin class such as `native-run`, `kubernetes-run`, `hpc-run`, or `remote-run`
- redacted bundles keep structure while replacing sensitive payloads irreversibly
- import verify-only mode must validate compatibility without materializing a replayed run

## Related tests

- `crates/bijux-dag-app/tests/run_dir_import_export_contract.rs`
- `crates/bijux-dev/tests/run_dir_import_export_hardening_contracts.rs`

## Versioning and change policy

Any incompatible change to export bundle versions, export modes, or provenance
field semantics must update this contract and the linked tests in the same
change.
