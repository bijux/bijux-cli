---
title: Run Dir Evolution Rulebook
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Run Dir Evolution Rulebook

Run-directory format evolution must preserve explicit support and refusal
classification instead of silent fallback.

## Scope

This rulebook governs the run-dir format surface backed by
`evidence/compat/run_dir/` and the artifact-index and run-manifest compatibility
lanes.

## Evolution rules

- current run-dir format fixtures must remain inspectable and classified as
  supported
- unsupported future run-dir formats must be rejected explicitly
- run-dir schema index defaults must stay aligned with compatibility lanes
- manifest version changes must remain distinguishable from run-dir format
  changes

## Related tests

- `crates/bijux-dag-app/tests/version_fixture_contracts.rs`
- `crates/bijux-dag-artifacts/tests/run_manifest_roundtrip_and_retention_contracts.rs`
- `crates/bijux-dev/tests/foundation_version_compatibility_lanes_contracts.rs`

## Versioning and change policy

Any incompatible run-dir format or supporting artifact-index change must update
this rulebook, the lane contract, and the related fixtures or tests in the
same change.
