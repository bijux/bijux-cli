# Bundles And Portability

## Purpose
Explain bundle export/import workflows and practical portability guarantees.

## Context
Bundles are the primary mechanism for moving reproducible workflow context across environments.

## Explanation
Bundle workflow:
1. export bundle from known run/graph context
2. transfer bundle to target environment
3. import bundle
4. replay or run validation in target

Portability guarantees (user-level):
- bundle format is intended for cross-environment handoff
- portability is bounded by supported backend/runtime constraints

What portability does not mean:
- identical performance across all environments
- universal feature parity across unsupported backends

End-to-end workflow example:
- author DAG
- execute run
- export bundle
- import bundle in target
- replay and diff against source baseline

## Examples
```bash
# Export bundle
bijux-dag bundle export --run-id RUN_20260309_120 --out ./exports/run120.bundle

# Import bundle
bijux-dag bundle import --bundle ./exports/run120.bundle

# Validate behavior after import
bijux-dag replay --run-id RUN_20260309_120
bijux-dag diff run --left RUN_20260309_120 --right RUN_20260309_121
```

## Guarantees
- Bundle workflow is documented as a concrete operational path.
- Portability boundaries are described explicitly, not implied.

## Limitations
- Portability remains constrained by backend support and environment contracts.
- This guide does not define low-level bundle schema internals.

## Related
- `docs/03-user-guide/05-replay.md`
- `docs/03-user-guide/06-diff.md`
- `docs/07-operations/05-backend-support.md`
- `docs/06-specification/03-artifact-model.md`
