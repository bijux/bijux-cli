# Sacred Execution Hardening Report

## Purpose

This report records the repository surfaces that currently harden the sacred
execution flow and keep engine centralization tied to executable proof.

## Guarded surfaces

- contract: `docs/spec/SACRED_EXECUTION_FLOW.md`
- architecture: `docs/bijux-dag/architecture/runtime-execution-flow.md`
- runtime hooks: `crates/bijux-dag-runtime/src/runtime_core/governance/sacred_execution.rs`
- engine: `crates/bijux-dag-runtime/src/runtime_core/execution/engine.rs`
- runtime tests: `crates/bijux-dag-runtime/tests/sacred_execution_flow_contracts.rs`
- maintainer guard: `crates/bijux-dev/tests/sacred_execution_hardening_contracts.rs`

## Current hardening stance

- engine flow must call the sacred hook layer for input materialization, cache,
  retry, trace, and dependency work
- direct helper bypasses in engine code are treated as contract violations
- documentation and tests must move together when hook ownership changes
