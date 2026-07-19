---
title: Ownership Boundary
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-04-06
---

# Ownership Boundary

Use this page when you need to understand where DAG responsibility really
changes hands: graph truth, runtime execution, operator-facing command
surfaces, and retained artifacts do not belong to one giant crate.

That split matters because most hard questions in `bijux-dag` are not just
"where is the code?" They are questions about which layer is allowed to define
truth.

## Start With The Boundary That Matters

| If the issue is about... | The first owner is... |
| --- | --- |
| whether the graph is valid, canonical, and deterministic | `bijux-dag-core` |
| how a valid graph executes, retries, replays, or reuses cached work | `bijux-dag-runtime` |
| how operators invoke the tool and read the results | `bijux-dag-app` and `bijux-dag-cli` |
| how run evidence is stored, hashed, and inspected | `bijux-dag-artifacts` plus runtime evidence writers |

## What Each Layer Owns

- `bijux-dag-cli` owns process entry and shell completion wiring only.
- `bijux-dag-app` owns command orchestration and response formatting.
- `bijux-dag-core` owns deterministic graph semantics and planner lowering.
- `bijux-dag-runtime` owns execution engine, scheduler, policy, and replay logic.
- `bijux-dag-artifacts` owns artifact formats, integrity, and lifecycle helpers.

## Reader Rules

- Keep graph-definition questions in the core layer until runtime side effects
  genuinely begin.
- Keep operator-facing summaries and diagnostics in the app layer rather than
  teaching the runtime crate to narrate product UX.
- Keep stored evidence rules explicit; do not smear them across app, runtime,
  and artifact docs until ownership becomes impossible to tell.

## Where The Boundary Is Enforced

- app crate boundary checks:
  `crates/bijux-dag-app/tests/crate_boundary_contract.rs`
- runtime boundaries: `crates/bijux-dag-runtime/tests/`
- core purity and identity behavior: `crates/bijux-dag-core/tests/`

## Code Anchors

- `crates/bijux-dag-app/docs/CONTRACTS.md`
- `crates/bijux-dag-core/docs/CONTRACTS.md`
- `crates/bijux-dag-runtime/docs/CONTRACTS.md`
- `crates/bijux-dag-artifacts/docs/CONTRACTS.md`

## Continue Reading

- [Release Boundary](release-boundary.md) for what the shipped `v0.4.0`
  product actually promises
- [DAG Packages](../packages/index.md) for crate-by-crate routing once you know
  the owning layer
- [Dependency Direction](../architecture/dependency-direction.md) for the
  technical constraints that keep these boundaries intact
