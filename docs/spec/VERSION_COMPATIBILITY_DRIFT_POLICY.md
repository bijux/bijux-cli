---
title: Version Compatibility Drift Policy
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Version Compatibility Drift Policy

Compatibility documentation, machine-readable lane contracts, and executable
fixtures must not drift apart.

## Scope

This policy governs alignment between:

- `contracts/foundation/version_compatibility_lanes.v1.json`
- `crates/bijux-dev/tests/data/foundation/version_compatibility_lanes_fixtures.json`
- `evidence/compat/`
- the compatibility and evolution docs in this repository

## Drift rules

- adding a new compatibility lane requires fixture and doc updates
- changing a current or previous version requires contract and fixture updates
- refused versions must remain listed and test-backed
- human-facing compatibility docs must not claim support that the lane
  contract refuses

## Related tests

- `crates/bijux-dev/tests/foundation_version_compatibility_lanes_contracts.rs`
- `crates/bijux-dag-app/tests/version_fixture_contracts.rs`

## Versioning and change policy

Any incompatible compatibility-lane change must update this policy, the lane
contract, and the linked fixtures or tests in the same change.
