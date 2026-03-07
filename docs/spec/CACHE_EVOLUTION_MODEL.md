# Cache Evolution Model

## Scope
This document defines cache key inputs, metadata compatibility, lineage, and verification expectations.

## Cache key input model
Intentional cache key inputs:
- node fingerprint
- adapter id + adapter version
- declared output schema version
- relevant runtime policy and config inputs

Accidental inputs are forbidden and must be treated as drift.

## Metadata compatibility
Cache metadata version is tracked independently from run-dir format.
Current expected cache metadata version: `cache-meta/v0.1`.

## Cache lineage model
Each cache entry should record:
- source run ID
- source node ID
- source node fingerprint
- creation timestamp
- cache source class (`local`, `imported-pack`, `remote-copy`)

## Verification requirements
- entries missing required proof metadata are invalid
- stale/unsupported metadata versions are rejected explicitly
- output proof hashes must match on verification

## Inspection surfaces
- `dag cache explain` for hit/miss causality
- `dag cache verify` for integrity walks
- `dag cache stats` for invalid-entry and size visibility
- `dag cache diff` for semantic comparison between cache entries

## Locality decision
Cache in this repository is local-first and filesystem-scoped.
Portable/promotable cache behavior is limited to explicit pack/unpack flows.
