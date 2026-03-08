# Cache hardening report

## Scope

Captures cache identity, proof verification, inspection surfaces, and governance evidence.

## Identity and invalidation

Intentional cache identity inputs:

- node fingerprint
- adapter id and adapter version
- output schema version
- policy fingerprint
- config fingerprint
- backend class

Cache invalidation is required for meaningful planner, backend, policy, and config changes.

## Proof and metadata verification

Required proof fields:

- `node_fingerprint`
- `adapter_id`
- `adapter_version`
- `cache_metadata_version`
- output proof index (`outputs/index.json`)

Unsupported, stale, truncated, or missing proof metadata fails closed.

## Inspection surfaces

Required cache inspection commands:

- `dag cache explain`
- `dag cache verify`
- `dag cache stats`
- `dag cache diff`

## Corruption evidence

Corruption fixtures remain mandatory for:

- missing metadata
- hash mismatch
- unsupported metadata version
- truncated metadata
- missing outputs proof

## Battle trust linkage

Cache hardening protects battle trust property `tp_cache_integrity` and remains mandatory release evidence.
