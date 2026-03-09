# Bundles And Portability

## Purpose
Explain bundle export/import workflows and portability boundaries with practical validation steps.

## Context
Bundles are the handoff mechanism for moving workflow context between environments.

## Explanation
Bundle workflow:
1. export bundle from a known run
2. transfer bundle to target environment
3. import bundle
4. replay and diff against baseline

Portability boundaries:
- portability means transferable workflow context, not universal backend parity
- behavior remains bounded by environment and support matrix constraints

End-to-end operator workflow:
```text
Author DAG -> Run -> Export bundle -> Import bundle -> Replay -> Diff -> Decision
```

This workflow is the recommended baseline for release confidence and migration checks.

## Examples
```bash
bijux-dag bundle export --run-id RUN_20260309_120 --out ./exports/run120.bundle
bijux-dag bundle import --bundle ./exports/run120.bundle
bijux-dag replay --run-id RUN_20260309_120
bijux-dag diff run --left RUN_20260309_120 --right RUN_20260309_121
```

## Guarantees
- Bundle flow is documented as a complete operational loop.
- Portability limits are explicit and non-speculative.

## Limitations
- Does not define low-level bundle schema internals.
- Does not guarantee identical performance across environments.

## Related
- `docs/03-user-guide/05-replay.md`
- `docs/03-user-guide/06-diff.md`
- `docs/07-operations/05-backend-support.md`
- `docs/06-specification/03-artifact-model.md`
