---
title: Module Map
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-19
---

# Module Map

Use this map to place a change before editing source. The package boundary says
which crate owns a fact; the internal boundary says which module owns its
implementation.

## Package Responsibilities

| Question | Owning package | Primary source boundary |
| --- | --- | --- |
| What does this graph mean? | `bijux-dag-core` | `crates/bijux-dag-core/src/graph/`, `crates/bijux-dag-core/src/pipeline/`, `crates/bijux-dag-core/src/analysis/`, and `crates/bijux-dag-core/src/planner/` |
| How is retained run evidence represented and protected? | `bijux-dag-artifacts` | `crates/bijux-dag-artifacts/src/storage/`, `crates/bijux-dag-artifacts/src/io/`, `crates/bijux-dag-artifacts/src/integrity/`, `crates/bijux-dag-artifacts/src/layout/`, and `crates/bijux-dag-artifacts/src/lifecycle/` |
| How is a validated plan executed or replayed? | `bijux-dag-runtime` | `crates/bijux-dag-runtime/src/runtime_core/`, `crates/bijux-dag-runtime/src/backend/`, `crates/bijux-dag-runtime/src/adapters/`, `crates/bijux-dag-runtime/src/policy/`, `crates/bijux-dag-runtime/src/cache/`, `crates/bijux-dag-runtime/src/replay/`, and `crates/bijux-dag-runtime/src/diagnostics/` |
| How does a user command coordinate those services? | `bijux-dag-app` | `crates/bijux-dag-app/src/commands/`, `crates/bijux-dag-app/src/graph/`, `crates/bijux-dag-app/src/inspect/`, `crates/bijux-dag-app/src/replay/`, `crates/bijux-dag-app/src/repair/`, `crates/bijux-dag-app/src/read/`, `crates/bijux-dag-app/src/write/`, and `crates/bijux-dag-app/src/format/` |
| How does the process start and return an exit status? | `bijux-dag-cli` | `crates/bijux-dag-cli/src/main.rs` |
| How do repository tests share deterministic evidence? | `bijux-dag-testkit` | its crate root and fixture builders |

## Core: Pure Graph Meaning

- The graph module owns domain types, canonicalization, and deterministic
  topology.
- The pipeline module owns parse, resolve, and validate entrypoints.
- The analysis module owns fingerprints and semantic analysis.
- The planner module owns lowering and plan construction.
- The build module owns authoring and compile-oriented wrappers, not alternate
  graph semantics.
- The contracts module owns core error and compatibility types.

Core algorithms do not belong in `lib.rs`, app handlers, or runtime adapters.
Core remains usable without filesystem, process, environment, or clock access.

## Artifacts: Format And IO Ownership

- Storage owns run evidence models, hardening, and persistence services.
- IO owns filesystem-backed reads and writes.
- Integrity owns hashes, indexes, proofs, and schema checks.
- Layout owns path construction and platform layout.
- Lifecycle owns lineage, retention, and promotion policy.

Runtime may orchestrate persistence, but artifact layout and integrity behavior
change here first.

## Runtime: Execution Effects

- Runtime core owns execution, planning bridges, state, and runtime invariants.
- Backend and adapters own process, container, and backend capability
  boundaries.
- Policy owns runtime evaluation and traces.
- Cache owns cache identity, validation, lookup, and lineage decisions.
- Replay owns replay verification and semantic-difference analysis.
- Diagnostics and error modules own runtime observations and classifications.
- Internal modules contain non-public control, analysis, identity, performance,
  testing, and workflow support.
- `crates/bijux-dag-runtime/src/simulated_platform.rs` quarantines modeled
  surfaces that are not part of the stable runtime root.

The runtime may consume core and artifact contracts. It must not duplicate
their semantic authorities.

## App And Process Surfaces

`bijux-dag-app` turns typed inputs into calls across core, runtime, and
artifacts. Its focused domains own graph commands, execution commands,
inspection, replay, repair, migration, explanation, and rendering. `routes/`
exposes those application services; it is not a home for runtime algorithms.

`bijux-dag-cli` should remain small enough that startup and exit mapping are
obvious. If a change needs domain tests rather than process-wiring tests, it
almost certainly belongs below app, runtime, artifacts, or core.

## Placement Review

Before adding a module, identify:

1. the fact or effect it owns;
2. the lowest package that can own it without importing a higher layer;
3. whether it is pure semantics, retained-evidence IO, execution effect, or
   command orchestration;
4. the focused contract test that will fail if the boundary drifts.

Do not add a parallel helper in a higher crate to avoid changing the actual
owner. That creates two authorities and makes replay, compatibility, and error
behavior harder to reason about.

## Authorities

- package manifests and package-local contracts in each DAG crate
- repository package boundary:
  `contracts/foundation/workspace_package_boundary.v1.json`
- dependency contracts:
  `crates/bijux-dev/tests/foundation_dag_dependency_direction_contracts.rs`

## Next Reads

- [Dependency Direction](dependency-direction.md)
- [Code Navigation](code-navigation.md)
- [Public Imports](../interfaces/public-imports.md)
