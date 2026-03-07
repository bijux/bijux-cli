# Sacred Execution Flow

## Canonical pipeline

The runtime sacred flow is:

1. plan
2. schedule
3. execute
4. collect
5. persist
6. advance

Expanded checkpoint sequence:

1. validate graph and contracts
2. lower graph to execution plan
3. initialize `ExecutionContext` and scheduler state
4. compute dependency readiness
5. materialize declared node inputs
6. compute node fingerprint and cache lookup
7. execute adapter with centralized retry logic
8. collect node result and classify terminal status
9. write trace and attempt events
10. propagate failure/skip/cached outcomes deterministically
11. write cache on eligible success paths
12. finalize manifest and artifact indexes
13. advance run state to terminal outcome

## Canonical context and result models

- Run-scoped context: `execution_context::ExecutionContext`
- Node-scoped context: `execution_context::NodeExecutionContext`
- Canonical node result: `node_result::NodeResult`

## Sacred centralized hooks

- retry logic: `sacred_execution::run_retry_logic`
- failure propagation: engine policy branch handling
- artifact materialization: `sacred_execution::run_materialize_inputs`
- cache read/write: `sacred_execution::run_cache_lookup` / `sacred_execution::run_cache_write`
- readiness/dependency: `sacred_execution::resolve_dependencies` and `ready_queue_from_dependencies`
- trace writing: `sacred_execution::run_write_trace`

## Side-channel execution prohibition

- Runtime node execution must not bypass sacred hooks for retry, cache, trace, and dependency readiness.
- Direct cache/trace wiring in engine code is forbidden when a sacred hook exists.

## Failure-injection expectations

- Sacred flow checkpoints have failure-injection tests proving deterministic failure handling.
- Failure-injection evidence is tracked in runtime sacred-flow contract tests and foundation hardening reports.

## State machine guards

- run transitions use `state_machine::run_transition_allowed`
- node transitions use `state_machine::node_transition_allowed`
- invariant and verify surfaces reject illegal terminal accounting

## Replay path contract

Replay must call `Runtime::run` over replay snapshot graph and therefore share the same engine path and state transition rules.
