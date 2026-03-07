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
