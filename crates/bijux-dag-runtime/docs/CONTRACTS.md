# bijux-dag-runtime Contracts

Responsibility: Execution engine, scheduler behavior, policy enforcement, replay semantics, and runtime diagnostics.

## Responsibility
`bijux-dag-runtime` owns execution planning, runtime policy enforcement, node execution orchestration, trace emission, and artifact persistence orchestration.

## Internal boundaries
- `../src/runtime_core/`: orchestration boundary (`engine`, `scheduler`,
  state, planner bridge, execution context, invariants).
- `../src/backend/`: backend contract, local process, fake backend, capability descriptors.
- `../src/artifacts/`: authoritative run artifact writing and verification boundary.
- `../src/replay/`: replay verification, semantic diff, and explain surfaces.
- `../src/diagnostics/`: invariant/event/timeline diagnostics surfaces.
- `../src/error/`: runtime error code and classification boundary.
- `../src/cache/`: cache identity, proof validation, store and lineage surfaces.
- `../src/policy/`: policy evaluation and policy trace surfaces.
- `../src/adapters/`: adapter registry, adapter contract, and built-in adapter facade.
- `../src/internal/`: non-public selectors/clock/io boundary wrappers.
- `api`/`config`: public runtime surfaces.
- `policy`: policy modeling and evaluation helpers.
- `run_context`: execution context and dependency injection.
- `node_result`: node execution outcomes and attempt records.
- `trace`: trace writing boundary.
- `registry`: adapter registry boundary.
- `selectors`: selector parsing and filtering.
- `cache`: cache read/write orchestration.
- `builtins`: built-in adapter implementations.
- `../src/simulated_platform.rs`: explicit quarantine facade for modeled
  platform, distributed, and product surfaces retained for evidence and
  contract coverage but excluded from the stable runtime root.

## Effect boundary
Runtime must isolate subprocess creation behind explicit boundary helpers and avoid hidden ambient reads.

## Related schemas

- `configs/dag/schema/runtime_config.schema.json`
