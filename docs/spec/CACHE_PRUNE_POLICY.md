---
title: Cache Prune Policy
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Cache Prune Policy

Cache pruning must remain explicit, inspectable, and simulation-friendly before
destructive cleanup happens.

## Scope

This policy covers cache inspection, verification, statistics, and prune
simulation surfaces exposed through the `cache` command family.

## Prune rules

- prune simulation must be available before destructive cleanup
- prune decisions must preserve proof-carrying cache entries until retention
  policy makes them eligible
- corrupt entries must be visible to verification rather than silently ignored

## Related tests

- `crates/bijux-dag-app/tests/cache_evolution_contract.rs`
- `crates/bijux-dev/tests/cache_hardening_contracts.rs`

## Versioning and change policy

Any incompatible change to cache prune semantics or operator prune visibility
must update this policy and the linked tests in the same change.
