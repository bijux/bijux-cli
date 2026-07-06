# Sacred Execution Flow

## Scope

This contract defines the centralized execution hooks that must govern runtime
materialization, cache lookup, retry execution, trace writing, cache writes,
and dependency resolution.

## Sacred hooks

The execution engine must route these steps through
`crates/bijux-dag-runtime/src/runtime_core/governance/sacred_execution.rs`:

- `run_materialize_inputs`
- `run_cache_lookup`
- `run_retry_logic`
- `run_write_trace`
- `run_cache_write`
- `resolve_dependencies`

## Engine boundary

The engine may orchestrate the execution order, but it must not bypass the
sacred hook layer with direct calls to cache or trace helpers.

## Related tests

- `crates/bijux-dag-runtime/tests/sacred_execution_flow_contracts.rs`
- `crates/bijux-dev/tests/sacred_execution_hardening_contracts.rs`

## Versioning and change policy

Sacred hook names, ordering intent, and engine-bypass prohibitions are stable
contract surfaces. Any incompatible change requires updating this document and
the linked runtime and maintainer tests in the same change.
