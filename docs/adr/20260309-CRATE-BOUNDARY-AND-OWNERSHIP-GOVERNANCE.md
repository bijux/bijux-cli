# Crate Boundary and Ownership Governance

Status: accepted
Owner: architecture maintainers
Date: 2026-03-09

## Decision
Each crate has explicit boundary, ownership, and dependency constraints. Cross-crate drift is governed by contract checks.

## Consequences
- Public API and dependency contracts are enforced by policy.
- New modules must align with declared ownership classes.

## Merged Decision Record
This ADR is standalone. The historical decision text merged into this record is included below.

### SOURCE: 20260309-CRATE-BOUNDARY-GOVERNANCE.md
# Crate Boundary ADR

## Decision
Keep current crate set and enforce boundary contracts through `bijux-dev-dag` governance suites.

## Accepted boundaries
- runtime does not depend on app or cli crates
- app does not host scheduler/adapter execution internals
- cli remains thin and delegates command behavior to app
- artifact models and storage contracts remain in artifacts crate

## Service interface rule
- runtime exposes typed service interfaces for execution and artifact persistence boundaries
- artifacts exposes persistence service interfaces for run-dir operations

## Non-decisions
- No crate collapse at this time.
- No public stable runtime embedding API commitment beyond documented surfaces.

## Consequences
- Feature work must update boundary docs, tests, and governance checks in the same change.
- New dependency edges require explicit policy update and ownership rationale.

### SOURCE: 20260309-APP-CRATE-BOUNDARY.md
# App crate boundary decision

## Decision

`bijux-dag-app` remains a single crate for now.

## Rationale

- Command orchestration and output formatting share tight response-model contracts.
- Splitting into `runbook` or standalone `commands` crates would currently duplicate
  graph/runtime wiring and increase compatibility burden.
- Current module split (`commands`, `format`, `read`, `write`, `explain`, `graph`,
  `cache`, `replay`, `migrate`) provides boundary clarity inside one crate.

## Trigger to revisit

Revisit split when one of these is true:

- command families need independent release cadence
- app crate compile times materially regress due to command growth
- API stability policy requires separate crate-level versioning for command surfaces

### SOURCE: 20260308-ARTIFACT-CRATE-SCOPE-RUNTIME-APP-BOUNDARIES.md
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

### SOURCE: 0010-APP-CRATE-STRUCTURE-DECISION.md
# ADR 0010: Keep `bijux-dag-app` as a single orchestration crate with internal command modules

## Status
Accepted

## Date
2026-03-07

## Context
The current workspace has a dedicated binary crate (`bijux-dag-cli`) and an application crate (`bijux-dag-app`) that contains command parsing, dispatch, and orchestration logic. The proposed alternative is a split into an additional crate such as `bijux-dag-runbook` or `bijux-dag-commands`.

We need to reduce boundary churn while repairing module ownership and testability.

## Decision
Keep `bijux-dag-app` as a single crate and split it into explicit internal modules:
- `commands`
- `format`
- `read`
- `write`
- `explain`
- `graph`
- `cache`
- `replay`
- `migrate`

Do not introduce a second app-level crate now.

## Rationale
- Keeps dependency graph stable while boundary contracts are being enforced.
- Avoids duplicating command model types across crates.
- Preserves a thin `bijux-dag-cli` binary crate with wiring only.
- Allows stronger internal visibility control using `pub(crate)` defaults.

## Consequences
- `bijux-dag-app` must stay orchestration-only and avoid low-level runtime internals.
- Clap command structures must live under `src/commands/`.
- Any future split into multiple app crates requires a new ADR with measured API and maintenance impact.
