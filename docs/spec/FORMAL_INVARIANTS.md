---
title: Formal Invariants
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Formal Invariants

This specification records the invariant ids that protect graph meaning, plan
determinism, scheduler correctness, run evidence consistency, cache proof
compatibility, and artifact reference integrity in `bijux-dag`.

## Scope

This document governs the invariant registry in
`crates/bijux-dag-runtime/src/runtime_core/governance/invariants.rs`, the unit
coverage in `crates/bijux-dag-runtime/src/internal/testing/invariants_tests.rs`,
and the generated-shape proof in
`crates/bijux-dag-runtime/tests/formal_invariant_property_contracts.rs`.

## Invariant catalog

| Invariant id | Statement | Primary proof surface |
| --- | --- | --- |
| `INV-GRAPH-SHAPE-001` | Canonical graph shape stays acyclic, unique, and deterministic for equivalent inputs. | `crates/bijux-dag-core/tests/graph_identity_property_contracts.rs`, `crates/bijux-dag-runtime/tests/formal_invariant_property_contracts.rs` |
| `INV-PLAN-SHAPE-001` | Lowered execution plan shape stays deterministic for equivalent graph structure and runtime options. | `crates/bijux-dag-runtime/tests/formal_invariant_property_contracts.rs`, `crates/bijux-dag-runtime/tests/planner_lowering_contracts.rs` |
| `INV-SCHED-READY-001` | Ready-queue admission does not duplicate a node in one valid scheduler state. | `crates/bijux-dag-runtime/tests/scheduler_contract.rs`, `crates/bijux-dag-runtime/tests/concurrency_contracts.rs` |
| `INV-RUN-COUNTS-001` | Manifest node totals match observed terminal trace totals. | `crates/bijux-dag-runtime/src/internal/testing/invariants_tests.rs`, `crates/bijux-dag-app/src/inspect/integrity_service.rs` |
| `INV-TRACE-TIME-001` | Node trace completion time is not earlier than node trace start time. | `crates/bijux-dag-runtime/src/internal/testing/invariants_tests.rs`, `crates/bijux-dag-app/src/inspect/integrity_service.rs` |
| `INV-CACHE-PROOF-001` | Cache reuse requires metadata and proof compatibility. | `crates/bijux-dag-runtime/tests/cache_contracts.rs`, `crates/bijux-dag-app/tests/cache_evolution_contract.rs` |
| `INV-ARTIFACT-REF-001` | Artifact references resolve to durable outputs that remain attributable and inspectable. | `crates/bijux-dag-app/tests/app_smoke_routed_workflows_contract.rs`, `crates/bijux-dag-app/tests/artifact_import_corruption_contract.rs` |
| `INV-RUN-TERMINAL-001` | Completed runs include at least one terminal node status. | `crates/bijux-dag-runtime/src/internal/testing/invariants_tests.rs`, `crates/bijux-dag-app/src/inspect/integrity_service.rs` |
| `INV-PLAN-DEPENDENCY-001` | Planned dependency counts remain stable for equivalent lowered graphs. | `crates/bijux-dag-runtime/tests/planner_lowering_contracts.rs`, `crates/bijux-dag-runtime/tests/runtime_execution_helper_expansion_contracts.rs` |

## Interpretation notes

- `INV-GRAPH-SHAPE-001` constrains canonical graph structure before runtime
  dispatch begins.
- `INV-PLAN-SHAPE-001` and `INV-PLAN-DEPENDENCY-001` constrain lowering and
  dependency accounting after graph validation succeeds.
- `INV-SCHED-READY-001`, `INV-RUN-COUNTS-001`, and `INV-TRACE-TIME-001`
  constrain live runtime state and verification output.
- `INV-CACHE-PROOF-001` and `INV-ARTIFACT-REF-001` constrain operator-visible
  reuse and inspection surfaces.

## Related tracking

- `docs/reports/governance/INVARIANT_COVERAGE.md`
- `docs/bijux-dag/quality/invariants.md`

## Versioning and change policy

Every normative invariant statement in this file cites an `INV-...` id. Any
incompatible invariant change must update this file, the runtime registry, and
the linked proof surfaces in the same change.
