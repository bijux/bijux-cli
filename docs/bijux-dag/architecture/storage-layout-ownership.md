---
title: Storage Layout Ownership
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Storage Layout Ownership

Artifact and cache path ownership is centralized so runtime code does not drift
into ad hoc filesystem behavior.

## Ownership boundary

- `ArtifactStore` owns validated run-dir writes and reads
- `CacheStore` owns cache entry metadata layout
- `engine.rs` may coordinate storage through approved helpers
- other runtime modules must not hardcode `manifest.json`,
  `outputs.index.json`, or direct `staging_path().join("nodes")` access
- read-only diagnostics, control-audit, and internal test modules may inspect
  finalized storage files without becoming storage layout owners

## Code anchors

- `crates/bijux-dag-runtime/src/artifacts/storage/store.rs`
- `crates/bijux-dag-runtime/tests/storage_contracts.rs`
- `crates/bijux-dag-runtime/src/runtime_core/execution/engine.rs`
