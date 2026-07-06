---
title: Migration Policy
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Migration Policy

Compatibility-breaking changes in `bijux-dag` require explicit migration
evidence, not silent format drift.

## Scope

This policy covers migration-oriented fixtures under `evidence/compat/migrations/`
for graph, run, artifact, and proof surfaces.

## Migration rules

- supported migrations must be backed by source fixtures
- a migration note must name the source surface and the target supported lane
- unsupported versions must remain refused unless a real migration path is
  added and documented

## Related tests

- `crates/bijux-dev/tests/foundation_version_compatibility_lanes_contracts.rs`
- `crates/bijux-dag-artifacts/tests/run_manifest_roundtrip_and_retention_contracts.rs`

## Versioning and change policy

Any new migration path or removed migration path must update this policy and
the supporting compatibility fixtures in the same change.
