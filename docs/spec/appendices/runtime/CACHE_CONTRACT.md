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
