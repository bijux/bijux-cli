# Cache Contract

## Scope
Defines cache key inputs, cache invalidation semantics, and correctness behavior.

## Invariants
- Cache key behavior is fingerprint-driven.
- Cache modes are explicit: off, read, read-write.
- Cache hit behavior must preserve semantic output equivalence.

## Related tests
- `tests/e2e/cache/*`
- `crates/bijux-dag-app/tests/e2e_integration_scenarios.rs`

## Related schemas
- `configs/schema/run_manifest.schema.json`

## Versioning and change policy
Any key-space change must be documented and covered by cache invalidation tests.
