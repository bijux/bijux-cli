# CACHE AND STORAGE CONTRACTS

Status: stable
Audience: maintainers
Owner: platform documentation guild

This standalone specification consolidates all relevant contract material for this domain.

## SOURCE: docs/spec/CACHE_CONTRACT.md
# Superseded by runtime cluster contract

- Superseded by: [RUNTIME_EXECUTION_AND_SCHEDULER_CONTRACT.md](./RUNTIME_EXECUTION_AND_SCHEDULER_CONTRACT.md)
- Appendix source: [appendices/runtime/CACHE_CONTRACT.md](./appendices/runtime/CACHE_CONTRACT.md)

## SOURCE: docs/spec/CACHE_EVOLUTION_MODEL.md
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

## SOURCE: docs/spec/CACHE_PRUNE_POLICY.md
# Cache Prune Policy

## Current policy
- Safe mode: remove only entries that fail verification.
- Simulation mode: report candidates without mutating cache.

## Future policy candidates
- age-based pruning
- size-budget pruning
- recency + verification score pruning

No future policy is normative until implemented and contract-tested.

## SOURCE: docs/spec/CACHE_SEMANTICS.md
# Superseded by runtime cluster contract

- Superseded by: [RUNTIME_EXECUTION_AND_SCHEDULER_CONTRACT.md](./RUNTIME_EXECUTION_AND_SCHEDULER_CONTRACT.md)
- Appendix source: [appendices/runtime/CACHE_SEMANTICS.md](./appendices/runtime/CACHE_SEMANTICS.md)

## SOURCE: docs/spec/CACHE_SYSTEM_INTEGRITY_CONTRACT.md
# Cache System Integrity Contract

## Purpose

Define required cache behavior guarantees for deterministic keys, correct invalidation, integrity verification, concurrency safety, retention discipline, telemetry, and explainability integration.

## Required cache behavior coverage

- deterministic cache key generation and lookup consistency
- invalidation correctness for graph, environment, artifact, and replay-ancestry changes
- cache integrity verification and corruption detection
- cache concurrency safety and eviction correctness
- cache retention and lifecycle consistency
- cache stress and performance benchmark coverage
- cache telemetry and explainability integration

## Required governance artifacts

- cache integrity regression corpus
- cache integrity verification suite
- cache integrity benchmark and telemetry reports
- cache lifecycle and retention reports
- cache explainability integration report

## SOURCE: docs/spec/PATH_NORMALIZATION_POLICY.md
# Path normalization policy

Applies to DAG output paths, input materialization paths, and artifact index paths.

Rules:
- Paths must be relative.
- Absolute paths are invalid.
- Parent traversal (`..`) is invalid.
- Backslash separators are normalized to slash separators for canonical comparison.
- Canonicalization must preserve deterministic ordering independent of OS path separator representation.

## SOURCE: docs/spec/STORAGE_CONTRACT.md
# Storage Contract

## Scope

Defines persistence ownership boundaries for run artifacts, cache metadata,
export bundles, and storage verification surfaces.

## Current Backend Support

- Supported storage backend today: filesystem
- No alternate storage backend is implemented in runtime execution path.
- Future backend discussion belongs to architecture docs only.

## Persisted Artifacts and Owners

| Artifact | Path | Owner module |
| --- | --- | --- |
| run manifest | `manifest.json` | runtime engine + artifacts contract |
| node traces | `nodes/<node>/trace.json` | runtime trace writer |
| outputs index | `outputs.index.json` | artifacts index writer |
| run snapshot | `run.snapshot.json` | runtime engine |
| run attempts | `run.attempts.json` | runtime engine |
| cache meta | `cache/<key>/meta.json` | runtime cache store |
| cache outputs index | `cache/<key>/outputs/index.json` | runtime cache store |

## Storage API Boundaries

- Run metadata persistence uses `ArtifactStore`.
- Cache metadata persistence uses `CacheStore`.
- Storage path validation is centralized in `validate_storage_relative_path`.

## Path and Traversal Rules

- Absolute paths are rejected for storage-relative API calls.
- `..` traversal segments are rejected.
- Backslash-delimited traversal is rejected.

## Atomic Write Rules

Critical metadata writes use temp-file plus rename policy:

- write to `*.tmp`
- rename to final path

Required for:

- run-side JSON metadata written through storage API
- cache metadata (`meta.json`)

## Partial Write and Corruption Handling

- Run manifest reader validates parse and required fields (`run_id`).
- Cache metadata reader validates parse and required field (`fingerprint`).
- Storage health report flags missing or invalid manifest/index surfaces.

## Run Directory Reader Contract

`ArtifactStore::read_validated_run_manifest` must fail if:

- manifest is missing
- manifest JSON is invalid
- required contract fields are absent

## Cache Reader Contract

`CacheStore::read_validated_cache_meta` must fail if:

- metadata file is missing
- metadata JSON is invalid
- required fingerprint metadata is missing

## Migration Policy

- No automatic storage migration is implemented.
- Existing layout compatibility is maintained by strict readers and contract docs.

## Verifying Surfaces

- `crates/bijux-dag-runtime/src/store.rs`
- `crates/bijux-dag-runtime/tests/storage_contracts.rs`
- `bijux-dev-dag storage-health`
- `bijux-dev-dag repo` suite `storage-boundaries`

## SOURCE: docs/spec/appendices/runtime/CACHE_CONTRACT.md
# Cache contract

## Scope

Defines cache identity, proof requirements, lineage metadata, verification behavior, and governance limits.

## Cache identity inputs

Intentional cache key inputs:

- node fingerprint
- adapter id
- adapter version
- output schema version
- policy fingerprint
- config fingerprint
- backend class

Accidental inputs are forbidden and treated as drift.

## Proof model

Required proof fields per cache entry:

- `node_fingerprint`
- `adapter_id`
- `adapter_version`
- `cache_metadata_version`
- outputs proof index under `outputs/index.json`

Entries missing required proof fields or outputs proof are invalid.

## Metadata version

- Cache metadata version is independent from run-directory format.
- Current supported metadata version: `cache-meta/v0.1`.
- Stale or truncated metadata must fail closed.

## Lineage model

Cache entry metadata must include lineage context where available:

- `source_run_id`
- `source_node_id`
- `cache_source` (`local`, `imported-pack`, or `remote-copy`)

## Operator surfaces

Required cache surfaces:

- `dag cache explain`
- `dag cache verify`
- `dag cache stats`
- `dag cache diff`

## Correctness expectations

- Cache key stability for semantically identical inputs.
- Cache key invalidation on planner-significant changes.
- Cache key invalidation on backend capability changes.
- Cache key invalidation on policy/config changes.
- Cache hit semantics equivalent to fresh execution for covered scenarios.

## Governance

Cache feature expansion is blocked unless proof verification coverage expands in the same change.

## SOURCE: docs/spec/appendices/runtime/CACHE_SEMANTICS.md
# Cache semantics

## Cache modes

- `off`: never read/write cache entries.
- `read`: read cache entries when valid; do not write new entries.
- `readwrite`: read cache when valid and write missing/updated cache entries.

## Cache proof contract

Each node trace may include proof fields indicating:
- `hit`: whether a cache entry was used.
- `corrupt_detected`: whether cached data integrity failed and was rejected.
- `validated`: whether fingerprint/provenance checks passed.

## Repair expectations

- Corrupt entries must be detected and recomputed before returning success.
- Cache directory must remain deterministic for repeated runs under unchanged inputs and policies.
- Offline cache checks can fail closed in `read` mode; successful runs must recompute invalid entries.
