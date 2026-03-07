# Sacred Execution Flow

## Canonical pipeline
1. validate graph and contracts
2. lower graph to execution plan
3. initialize run context and scheduler state
4. compute dependency readiness
5. materialize declared node inputs
6. compute node fingerprint and cache lookup
7. execute adapter with centralized retry logic
8. write trace and attempt events
9. propagate failure/skip/cached outcomes deterministically
10. write cache on eligible success paths
11. finalize manifest and artifact indexes

## Sacred centralized hooks
- retry logic: `sacred_execution::run_retry_logic`
- failure propagation: engine policy branch handling
- artifact materialization: `sacred_execution::run_materialize_inputs`
- cache read/write: `sacred_execution::run_cache_lookup` / `sacred_execution::run_cache_write`
- readiness/dependency: `sacred_execution::resolve_dependencies` and `ready_queue_from_dependencies`
- trace writing: `sacred_execution::run_write_trace`

## State machine guards
- run transitions use `state_machine::run_transition_allowed`
- node transitions use `state_machine::node_transition_allowed`
- invariant and verify surfaces reject illegal terminal accounting

## Replay path contract
Replay must call `Runtime::run` over replay snapshot graph and therefore share the same engine path and state transition rules.
