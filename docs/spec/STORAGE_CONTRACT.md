---
title: Storage Contract
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Storage Contract

`bijux-dag-runtime` writes run-dir and cache artifacts through explicit storage
helpers instead of ad hoc path joins scattered across runtime modules.

## Scope

This contract covers the storage surface in
`crates/bijux-dag-runtime/src/artifacts/storage/store.rs` and the storage
conformance tests in `crates/bijux-dag-runtime/tests/storage_contracts.rs`.

## Owned storage surfaces

Stable storage helpers are:

- `ArtifactStore`
- `CacheStore`
- `StorageHealthReport`
- `validate_storage_relative_path`

Runtime modules outside approved storage and engine surfaces must not hardcode
manifest, outputs index, or node staging paths.

Read-only diagnostics, control-audit, and internal test surfaces may inspect
finalized storage files when they do not become new write authorities.

## Run-dir rules

- run manifests are read through validated storage helpers
- writes to run-dir staging paths use explicit helper methods
- storage-relative paths must reject traversal, absolute paths, and backslash
  escapes
- rooted input and output authorization must reject paths that escape the
  governed input or output tree
- missing `outputs/index.json` is a storage-health anomaly, not a silent success
- storage validation protects governed write paths; it does not claim a general
  host filesystem sandbox for shell execution

## Cache rules

- cache entry metadata writes are atomic
- cache entry keys must satisfy the same storage-relative path validation rules
- cache metadata must include the retained cache key and proof-compatible
  identity inputs used by cache verification

## Related tests

- `crates/bijux-dag-runtime/tests/storage_contracts.rs`
- `crates/bijux-dag-runtime/tests/security_model_contracts.rs`

## Versioning and change policy

Any incompatible change to storage path validation, run-dir helper ownership, or
cache metadata validation must update this contract and the linked tests in the
same change.
