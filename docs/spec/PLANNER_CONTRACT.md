# Planner contract

## Authority

This document is the single normative authority for planner inputs, lowering stages, outputs, and guarantees.

## Boundary model

- parsed graph: strict parse output
- validated graph: parse output after schema and semantic validation
- canonical graph: deterministic normalization for identity and ordering
- execution plan: lowered runtime representation

## Lowered plan model

Execution plan uses lowered structures only:

- `PlannedNode`: execution-relevant fields (`id`, `kind`, `deps`, `outputs`, `retry`, `timeout_ms`)
- `PlannedDependency`: lowered dependency edge (`from`, `to`)

Planner boundary owns graph lowering; runtime execution consumes lowered plan semantics.

## Fingerprints

- `graph_fingerprint`: canonical graph identity
- `planner_fingerprint`: lowered plan identity

These fingerprints have distinct meaning and must not be conflated.

## Determinism guarantees

- semantically equivalent graphs lower to identical planner fingerprints
- cosmetic metadata changes do not alter lowered semantic plan identity
- plan ordering is deterministic

## Validation and diagnostics

- schema/semantic validation errors are distinct from planner lowering errors
- planner diagnostics use stable IDs:
  - `P4000`: planner generic failure
  - `P4013`: unsupported node kind
  - `P4016`: warning for outputless execution node
  - `P4021`: runtime capability requirement rejected during lowering

## Selector and pruning stage

Selector pruning occurs after validation and before final lowering output is committed.

## Graph shape coverage

Planner lowering coverage includes:

- fan-in graphs
- fan-out graphs
- disconnected graphs (supported)

## Debug and schema surfaces

- debug command: `bijux-dev-dag dag plan-dump --graph <path>`
- schema: `configs/schema/execution_plan.schema.json`

## Required evidence

- `crates/bijux-dag-core/tests/planner_contract.rs`
- `crates/bijux-dag-runtime/tests/planner_lowering_contracts.rs`
- `planner-alignment` control-plane suite
