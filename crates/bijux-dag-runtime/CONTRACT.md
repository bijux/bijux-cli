# bijux-dag-runtime contract

## Responsibility
`bijux-dag-runtime` owns execution planning, runtime policy enforcement, node execution orchestration, trace emission, and artifact persistence orchestration.

## Internal boundaries
- `api`/`config`: public runtime surfaces.
- `policy`: policy modeling and evaluation helpers.
- `run_context`: execution context and dependency injection.
- `node_result`: node execution outcomes and attempt records.
- `trace`: trace writing boundary.
- `registry`: adapter registry boundary.
- `selectors`: selector parsing and filtering.
- `cache`: cache read/write orchestration.
- `builtins`: built-in adapter implementations.

## Effect boundary
Runtime must isolate subprocess creation behind explicit boundary helpers and avoid hidden ambient reads.
