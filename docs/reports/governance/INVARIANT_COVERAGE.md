---
title: Invariant Coverage
audience: maintainers
type: report
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Invariant Coverage

This ledger maps each runtime invariant id to the registry location, executable
proof, and operator-facing enforcement surface.

| Invariant id | Registry | Executable proof | Operator or verifier surface |
| --- | --- | --- | --- |
| `INV-GRAPH-SHAPE-001` | `crates/bijux-dag-runtime/src/runtime_core/governance/invariants.rs` | `crates/bijux-dag-core/tests/graph_identity_property_contracts.rs`, `crates/bijux-dag-runtime/tests/formal_invariant_property_contracts.rs` | graph canonicalization and strict parse validation |
| `INV-PLAN-SHAPE-001` | `crates/bijux-dag-runtime/src/runtime_core/governance/invariants.rs` | `crates/bijux-dag-runtime/tests/formal_invariant_property_contracts.rs`, `crates/bijux-dag-runtime/tests/planner_lowering_contracts.rs` | `build_plan` determinism and plan-shape comparison |
| `INV-SCHED-READY-001` | `crates/bijux-dag-runtime/src/runtime_core/governance/invariants.rs` | `crates/bijux-dag-runtime/tests/scheduler_contract.rs`, `crates/bijux-dag-runtime/tests/concurrency_contracts.rs` | `scheduler_invariants_hold` |
| `INV-RUN-COUNTS-001` | `crates/bijux-dag-runtime/src/runtime_core/governance/invariants.rs` | `crates/bijux-dag-runtime/src/internal/testing/invariants_tests.rs` | `verify_run` and integrity inspection |
| `INV-TRACE-TIME-001` | `crates/bijux-dag-runtime/src/runtime_core/governance/invariants.rs` | `crates/bijux-dag-runtime/src/internal/testing/invariants_tests.rs` | deep and strict trace verification |
| `INV-CACHE-PROOF-001` | `crates/bijux-dag-runtime/src/runtime_core/governance/invariants.rs` | `crates/bijux-dag-runtime/tests/cache_contracts.rs`, `crates/bijux-dag-app/tests/cache_evolution_contract.rs` | cache verify and replay classification |
| `INV-ARTIFACT-REF-001` | `crates/bijux-dag-runtime/src/runtime_core/governance/invariants.rs` | `crates/bijux-dag-app/tests/app_smoke_routed_workflows_contract.rs`, `crates/bijux-dag-app/tests/artifact_import_corruption_contract.rs` | artifact inspect and import verification |
| `INV-RUN-TERMINAL-001` | `crates/bijux-dag-runtime/src/runtime_core/governance/invariants.rs` | `crates/bijux-dag-runtime/src/internal/testing/invariants_tests.rs` | completed-run integrity verification |
| `INV-PLAN-DEPENDENCY-001` | `crates/bijux-dag-runtime/src/runtime_core/governance/invariants.rs` | `crates/bijux-dag-runtime/tests/planner_lowering_contracts.rs`, `crates/bijux-dag-runtime/tests/runtime_execution_helper_expansion_contracts.rs` | plan dependency accounting and scheduler explain output |

## Review rule

Coverage is incomplete if a registry id is absent from this table or if the
listed proof no longer exercises the cited invariant.
