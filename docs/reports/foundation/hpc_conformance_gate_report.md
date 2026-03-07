# HPC Conformance Gate Report

## Release gate

HPC support claims are blocked unless all HPC fixture and contract suites pass.

## Required fixtures

- `evidence/battle/fixtures/hpc/simple_equivalence.dag.json`
- `evidence/battle/fixtures/hpc/staged_input_equivalence.dag.json`
- `evidence/battle/fixtures/hpc/checkpointed_partial_replay.dag.json`
- `evidence/battle/fixtures/hpc/delayed_scheduler_state_propagation.json`
- `evidence/operator/fixtures/hpc/queue_rejection_explain.json`
- `evidence/operator/fixtures/hpc/preemption_explain.json`

## Required contract suites

- `crates/bijux-dag-runtime/tests/backend_cluster_contracts.rs`
- `crates/bijux-dev-dag/tests/hpc_adapter_contracts.rs`
- `crates/bijux-dev-dag/tests/hpc_adapter_release_contracts.rs`
