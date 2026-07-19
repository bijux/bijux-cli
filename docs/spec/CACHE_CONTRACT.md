---
title: Cache Contract
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Cache Contract

The cache surface in `bijux-dag` is a proof-carrying reuse boundary, not a
best-effort convenience layer.

## Scope

This contract covers cache key inputs, cache metadata proof requirements, cache
entry manifest compatibility, corruption handling, and operator cache commands.

## Required proof boundary

Cache entries must carry explicit proof for:

- `cache_key`
- `node_fingerprint`
- `node_definition_fingerprint`
- `declared_environment_fingerprint`
- `input_lineage_fingerprint`
- `adapter_id`
- `adapter_version`
- `policy_fingerprint`
- `execution_contract_fingerprint`
- `backend_class`
- `produces_outputs_schema_version`
- `cache_metadata_version`

## Corruption handling

Cache corruption must fail closed. The governed corruption fixtures are:

- `evidence/cache/corrupt/missing_meta.json`
- `evidence/cache/corrupt/hash_mismatch.json`
- `evidence/cache/corrupt/missing_manifest.json`
- `evidence/cache/corrupt/unsupported_metadata_version.json`
- `evidence/cache/corrupt/truncated_meta.json`
- `evidence/cache/corrupt/missing_outputs_proof.json`

## Related tests

- `crates/bijux-dag-runtime/tests/cache_contracts.rs`
- `crates/bijux-dag-runtime/tests/cache_evolution_contracts.rs`
- `crates/bijux-dag-app/tests/cache_evolution_contract.rs`
- `crates/bijux-dev/tests/cache_hardening_contracts.rs`

## Versioning and change policy

Any incompatible change to cache proof fields, metadata compatibility, or
corruption behavior must update this contract and the linked tests in the same
change.
