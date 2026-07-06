---
title: Cache Evolution Model
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Cache Evolution Model

Cache evolution in `bijux-dag` is governed by semantic reuse meaning, explicit
metadata compatibility, and corruption-aware verification.

## Scope

This model covers key formation, metadata version acceptance, cache lineage,
local and remote cache locality decisions, and operator-visible verification
surfaces.

## Intentional cache key inputs

Intentional cache key inputs are the fields that change semantic reuse
meaning:

- execution fingerprint
- node definition fingerprint
- declared environment fingerprint
- input lineage fingerprint
- adapter identity and version
- output schema version
- policy fingerprint
- execution contract fingerprint
- backend class

## Metadata compatibility

- current cache metadata versions must remain accepted
- unsupported cache metadata versions must fail verification explicitly
- cache entry manifest versions must remain separately governed from metadata
  versions

## Cache lineage model

Cache lineage follows the semantic input and execution boundary that produced a
reusable output set. Reuse is valid only when lineage and execution contract
proof stay compatible.

## Locality decision

Locality decision determines whether the runtime may reuse local cache,
fallback, or reject a remote or incompatible cache surface based on proof and
backend compatibility.

## Related tests

- `crates/bijux-dag-runtime/tests/cache_contracts.rs`
- `crates/bijux-dag-runtime/tests/cache_evolution_contracts.rs`
- `crates/bijux-dag-app/tests/cache_evolution_contract.rs`

## Versioning and change policy

Any incompatible change to key inputs, metadata compatibility, lineage
semantics, or locality decision behavior must update this model and the linked
tests in the same change.
