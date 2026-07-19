---
title: Planner Contract
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-04-28
---

# Planner Contract

`bijux-dag` lowers a validated graph into an execution plan before the runtime
decides scheduling, replay posture, or evidence handling.

## Scope

This contract defines the planner output that must stay stable across graph
validation, execution-plan schema, runtime lowering, and operator-facing plan
inspection surfaces.

This contract exists so three surfaces stay aligned:

- graph semantics in `bijux-dag-core`
- executable plan shape in `configs/dag/schema/execution_plan.schema.json`
- runtime consumption in `bijux-dag-runtime`

## Lowering pipeline

The planner is the boundary between four distinct states that must not be
blurred together:

- parsed graph: strict syntax and structural decoding from the input DAG file
- validated graph: graph semantics after validation has accepted the DAG
- canonical graph: deterministic normalized graph identity used for planning and fingerprints
- execution plan: lowered runtime-facing plan with ordered nodes, dependencies, diagnostics, and identity boundaries

The maintainer-facing `dag plan-dump` command must expose the execution plan
without bypassing these boundaries.

## What The Planner Must Preserve

The planner output is not a cosmetic summary. It must preserve the execution
facts that a runtime or auditor needs without reopening the source graph:

- graph fingerprint and planner fingerprint
- execution and evidence identity boundaries
- ordered planned nodes
- ordered planned edges
- node executor kind and semantic kind
- trigger rules
- retry and timeout policy
- declared side effects and resources
- branch decisions, default branch, and decision output
- edge identity, edge kind, and conditional decision labels
- branch reachability analysis
- diagnostics and omitted identity fields

## Planned Nodes

Each planned node must carry:

- `id`
- `kind`
- `executor_kind`
- `semantic_kind`
- `deps`
- `io_contract`
- `outputs`
- `side_effects`
- `retry`
- `trigger_rule`
- optional `timeout_ms`
- optional `resources`
- optional `branch`

`kind` names the execution adapter surface such as `const`, `shell`, or
`container`.

`semantic_kind` names the graph meaning such as `task` or `branch`.

Those fields must remain separate. A branch node may still execute through the
shell adapter, but it is not semantically the same thing as a normal shell task.

## Planned Edges

Each planned edge must carry:

- optional `id`
- `kind`
- optional `decision`
- `from`
- `from_port`
- `to`
- `to_port`

Edge kinds are:

- `data`
- `control`
- `conditional`

Conditional edges are only valid from branch decision outputs and must keep the
chosen decision label.

## Branch Paths

For each branch node and each declared decision, the planner must emit a static
path summary:

- `branch_node_id`
- `decision`
- `direct_targets`
- `reachable_nodes`

This is a planning contract, not runtime proof. It explains which nodes the
planner believes are reachable when a decision fires, before execution happens.

## Identity Boundaries

The planner keeps several identities separate on purpose:

- `graph_fingerprint`: canonical graph structure
- `planner_fingerprint`: plan shape for scheduling and dependency semantics
- `execution_fingerprint`: behavior-affecting execution inputs
- `evidence_fingerprint`: execution inputs plus operator-facing evidence metadata

The planner must also disclose which fields are intentionally omitted from
execution identity so the boundary stays reviewable.

## Validation Relationship

Validation happens before planning.

The planner may refuse a graph for runtime compatibility reasons, but it must
not silently reinterpret an invalid graph. Branch decisions, conditional edges,
and trigger rules must be validated before lowering.

## Planning diagnostics

Planner diagnostics are part of the stable proof surface, not incidental debug
strings.

- `P4021` means the graph requested a runtime capability that the selected node
  kind cannot satisfy during lowering.
- capability-oriented planner diagnostics must remain visible in `dag plan-dump`
  output and in runtime-facing plan evidence.
- diagnostic codes must stay stable enough to support contract tests and trust
  property review.

## Required Regression Proof

Any change to planner contract fields or planner identity rules must update:

- `configs/dag/schema/execution_plan.schema.json`
- planner contract tests in `crates/bijux-dag-core/tests/`
- runtime lowering tests in `crates/bijux-dag-runtime/tests/`
- app route tests when planner-facing JSON output changes

Operator-facing docs may describe a plan surface as authoritative only when they
cite `docs/spec/PLANNER_CONTRACT.md` directly.

## Related tests

- `crates/bijux-dag-core/tests/planner_contract.rs`
- `crates/bijux-dag-core/tests/planner_fixture_contracts.rs`
- `crates/bijux-dag-core/tests/planner_validation_edge_case_contracts.rs`
- `crates/bijux-dag-runtime/tests/planner_lowering_contracts.rs`
- `crates/bijux-dag-app/tests/plan_command_contract.rs`

## Versioning and change policy

Planner output shape, identity boundaries, and branch reachability semantics are
stable contract surfaces. Any incompatible change requires updating this
document, the execution plan schema, and the linked planner, runtime, and app
contract tests in the same change.
