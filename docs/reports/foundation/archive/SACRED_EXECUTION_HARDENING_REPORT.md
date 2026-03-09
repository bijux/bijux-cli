# Sacred execution hardening report

## Scope

Captures hardening evidence for the canonical runtime execution flow.

## Canonical flow

The runtime follows one controlled path:

- plan
- schedule
- execute
- collect
- persist
- advance

## Centralized hook coverage

The engine path is required to use sacred hooks:

- `sacred_execution::resolve_dependencies`
- `sacred_execution::run_materialize_inputs`
- `sacred_execution::run_cache_lookup`
- `sacred_execution::run_retry_logic`
- `sacred_execution::run_write_trace`
- `sacred_execution::run_cache_write`

## Context and result authority

- run-scoped context: `ExecutionContext`
- node-scoped context: `NodeExecutionContext`
- canonical execution result: `NodeResult`

## Side-channel prohibition

Runtime engine code does not permit direct side-channel cache/trace wiring when sacred hooks exist.

## Failure-injection and contract evidence

- sacred flow contract tests include deterministic failure behavior checks
- governance guard enforces required docs, code surfaces, and hook usage

## Release linkage

Sacred execution flow conformance is mandatory foundation evidence and remains required by repository verification.
