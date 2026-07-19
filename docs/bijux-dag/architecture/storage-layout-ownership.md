---
title: Storage Layout Ownership
audience: mixed
type: architecture
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-10
---

# Storage Layout Ownership

This page names which runtime surfaces own persisted storage layout decisions so
run directories, cache entries, manifests, and output indexes do not drift
through scattered ad hoc path logic.

## Ownership Boundary

The runtime storage boundary is implemented through explicit storage helpers,
not by letting each execution surface invent its own retained paths.

- `crates/bijux-dag-runtime/src/artifacts/storage/store.rs` owns run-dir and
  cache-store write helpers
- `crates/bijux-dag-runtime/src/artifacts/storage/path_authorization.rs` owns
  path validation and escape rejection
- engine and adapter execution surfaces may request governed paths, but they do
  not become new authorities for retained layout rules

## Persisted Layout Surfaces

The governed layout includes:

- run-level manifests and completion markers
- node-local trace, stdout, stderr, and outputs index surfaces
- declared output materialization paths
- cache metadata, cache proof inputs, and cache object layout
- storage-health inspection surfaces that verify retained evidence after a run

## Write Rules

- runtime modules outside the storage boundary must not hardcode retained
  manifest, outputs index, or cache metadata paths
- adapter execution may write only through paths resolved by governed storage
  helpers
- storage-relative path validation must reject traversal, absolute paths, and
  backslash escape forms before any write authority is handed to an adapter
- diagnostics may inspect finalized storage files, but inspection code does not
  become a write authority

## Read Rules

Read-side operator and diagnostics surfaces may reopen retained storage to
support inspect, replay, diff, and health checks, but they must treat the
persisted layout as repository-owned evidence rather than a caller-defined
filesystem convention.

## Related Authority

- [Storage Contract](../../spec/STORAGE_CONTRACT.md)
- [State and Persistence](state-and-persistence.md)
- [Runtime Execution Flow](runtime-execution-flow.md)
