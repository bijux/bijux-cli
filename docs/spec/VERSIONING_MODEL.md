---
title: Versioning Model
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Versioning Model

`bijux-dag` versioning is governed per compatibility surface, not by a single
blanket promise.

## Scope

This model covers version lanes for graph specifications, run manifests,
run-dir formats, artifact indexes, export and proof bundles, and CLI envelopes
as defined by `contracts/foundation/version_compatibility_lanes.v1.json`.

## Compatibility lanes

Each governed surface must classify versions into one of these lanes:

- `current`
- `previous`
- `refused`

The current compatibility lanes are maintained in:

- `contracts/foundation/version_compatibility_lanes.v1.json`
- `crates/bijux-dev/tests/data/foundation/version_compatibility_lanes_fixtures.json`

## Governed versioned surfaces

- graph schema and graph spec fixtures under `evidence/compat/graph_schema/`
- run-dir format fixtures under `evidence/compat/run_dir/`
- run manifest fixtures under `evidence/compat/run_schema/`
- export bundle fixtures under `evidence/compat/export_bundle/`
- proof bundle fixtures under `evidence/compat/proof_bundle/`

## Related tests

- `crates/bijux-dev/tests/foundation_version_compatibility_lanes_contracts.rs`
- `crates/bijux-dag-app/tests/version_fixture_contracts.rs`
- `crates/bijux-dag-artifacts/tests/run_manifest_roundtrip_and_retention_contracts.rs`

## Versioning and change policy

Any incompatible change to a governed surface must update this model, the
compatibility lane contract, and the linked fixtures or tests in the same
change.
