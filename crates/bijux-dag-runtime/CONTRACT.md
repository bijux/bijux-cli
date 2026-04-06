# bijux-dag-runtime contract

Responsibility: Execution engine, scheduler behavior, policy enforcement, replay semantics, and runtime diagnostics.

## Responsibility
`bijux-dag-runtime` owns execution planning, runtime policy enforcement, node execution orchestration, trace emission, and artifact persistence orchestration.

## Internal boundaries
- `runtime_core/*`: orchestration boundary (`engine`, `scheduler`, `state`, planner bridge, execution context, invariants).
- `backend/*`: backend contract, local process, fake backend, capability descriptors.
- `artifacts/*`: authoritative run artifact writing and verification boundary.
- `replay/*`: replay verification, semantic diff, and explain surfaces.
- `diagnostics/*`: invariant/event/timeline diagnostics surfaces.
- `error/*`: runtime error code and classification boundary.
- `cache/*`: cache identity, proof validation, store and lineage surfaces.
- `policy/*`: policy evaluation and policy trace surfaces.
- `adapters/*`: adapter registry, adapter contract, and built-in adapter façade.
- `internal/*`: non-public selectors/clock/io boundary wrappers.
- `api`/`config`: public runtime surfaces.
- `policy`: policy modeling and evaluation helpers.
- `run_context`: execution context and dependency injection.
- `node_result`: node execution outcomes and attempt records.
- `trace`: trace writing boundary.
- `registry`: adapter registry boundary.
- `selectors`: selector parsing and filtering.
- `cache`: cache read/write orchestration.
- `builtins`: built-in adapter implementations.
- `simulated_platform`: explicit quarantine facade for modeled platform/distributed/product surfaces that are retained for evidence and contract coverage but are not part of the stable runtime root.

## Effect boundary
Runtime must isolate subprocess creation behind explicit boundary helpers and avoid hidden ambient reads.
