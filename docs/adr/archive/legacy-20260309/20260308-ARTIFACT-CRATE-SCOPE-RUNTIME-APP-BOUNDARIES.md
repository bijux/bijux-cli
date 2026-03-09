# ADR: Artifact Crate Scope vs Runtime/App Boundaries

- Date: 2026-03-08
- Status: Accepted

## Context
The artifact crate holds deterministic artifact data contracts, integrity helpers, and store-facing abstractions used by runtime and app layers. Over time, ownership boundaries can drift and pull runtime/app business policy into artifact internals.

## Decision
- Keep `bijux-dag-artifacts` focused on:
  - artifact path/index/manifest contracts,
  - integrity and hardening primitives,
  - store abstraction and capability surfaces,
  - lineage/retention explain helpers.
- Keep runtime/app focused on:
  - command-family orchestration,
  - backend lifecycle policy and routing decisions,
  - user-facing rendering and command semantics.
- Enforce with direct tests, generated truth reports, and zero-coverage drift gates for artifact io/storage source files.

## Consequences
- Artifact behavior remains deterministic and testable as a reusable contract layer.
- Runtime/app changes cannot silently regress artifact io/store/hardening invariants.
- Release governance has explicit evidence for artifact integrity and fixture-backed coverage.
