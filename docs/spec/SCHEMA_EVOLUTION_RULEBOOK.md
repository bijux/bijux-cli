---
title: Schema Evolution Rulebook
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Schema Evolution Rulebook

Graph schema evolution in `bijux-dag` must preserve explicit compatibility
lanes and fixture-backed refusal behavior.

## Scope

This rulebook governs graph schema and graph spec evolution backed by
`evidence/compat/graph_schema/` and the graph-spec surface in
`contracts/foundation/version_compatibility_lanes.v1.json`.

## Evolution rules

- current graph schema versions must remain accepted by strict parsing
- accepted previous versions must be intentionally normalized or mapped
- refused versions must stay refused and fixture-backed
- compatibility behavior must be demonstrated by supported and unsupported
  graph schema fixtures

## Related tests

- `crates/bijux-dev/tests/foundation_version_compatibility_lanes_contracts.rs`
- `crates/bijux-dag-app/tests/version_fixture_contracts.rs`

## Versioning and change policy

Any incompatible graph schema or graph spec change must update this rulebook,
the compatibility lane contract, and the supporting fixtures in the same
change.
