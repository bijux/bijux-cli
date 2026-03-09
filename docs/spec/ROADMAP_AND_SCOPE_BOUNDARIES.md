# ROADMAP AND SCOPE BOUNDARIES

Status: stable
Audience: maintainers
Owner: platform documentation guild

This standalone specification consolidates all relevant contract material for this domain.

## SOURCE: docs/spec/ADVANCED_SEMANTICS_QUARANTINED_SURFACES.md
# Advanced Semantics Quarantined Surfaces

This page lists advanced semantics families that remain quarantined from default runtime UX and core kernel semantics.

## Quarantined families

- distributed control-plane and federation modeling
- AI/operator-assist workflow modeling
- workflow product abstraction modeling
- dataset/catalog semantic modeling
- cost optimization modeling

## Why quarantined

- No concrete default user-facing execution path (`user_facing_path=false`).
- No direct test authority in core runtime (`direct_test=false`).
- No fixture-backed executable ownership (`example_or_fixture=false`).
- Must satisfy lifecycle policy `expire-or-graduate` with owner and target date.

## Graduation requirements

- Concrete user path added with explicit contract tests.
- Fixture-backed examples and deterministic behavior checks.
- Reclassification from `speculative` to a retained category in governance policy.

## SOURCE: docs/spec/ADVANCED_SEMANTICS_RETAINED_SURFACES.md
# Advanced Semantics Retained Surfaces

This page lists advanced semantics families retained in `bijux-dag` and why they remain inside runtime scope.

## Retained families

- `kernel-relevant`
  - Example: `runtime_core/execution/run_state.rs`
  - Reason: controls deterministic terminal-state semantics and replay-safety boundaries.

- `runtime-relevant`
  - Example: `runtime_core/execution/scheduler.rs`
  - Reason: required for runtime scheduling invariants and execution-policy behavior.

- `adapter-relevant`
  - Example: `adapters/registry.rs`
  - Reason: required for adapter resolution determinism and capability contract selection.

## Retention criteria

- Has concrete user-facing runtime path.
- Has direct tests and fixture-backed evidence.
- Has explicit owner (`owner_repo: bijux-dag`) in governance policy.

## SOURCE: docs/spec/ADVANCED_SEMANTICS_SCOPE.md
# Advanced Semantics Scope

Advanced semantics are not part of the core mission unless proven through executable paths and ownership.

Why quarantined:
- deterministic DAG kernel and replay proof semantics must remain stable
- speculative or platform-expansion modeling must not leak into default operator surfaces
- quarantine allows exploration without changing shipped trust boundaries

Governance sources:
- `configs/policy/advanced_semantics_governance.json`
- `configs/policy/runtime_scope_v2.json`
- `docs/reports/foundation/ADVANCED_SEMANTICS_INVENTORY.md`

## SOURCE: docs/spec/BATTLE_WORKFLOW_CONTRACT.md
# Battle workflow contract

## Scope

Battle workflows are executable stress scenarios for runtime behavior under realistic pressure.

## Scenario catalog

Fixtures live in `evidence/battle/workflows/runtime` and are validated by `battle_workflow_harness_contracts.rs`.

## Required scenarios

- medium workflow
- failure-heavy workflow
- artifact-heavy workflow
- cache invalidation workflow
- replay divergence workflow
- scheduler fairness workflow
- import/export workflow
- corruption workflow
- operator inspection workflow
- large dag workflow
- resource contention workflow
- multi-root workflow
- branch/join workflow
- retry storm workflow
- timeout workflow
- version compatibility workflow
- malformed run-dir workflow
- ugly realistic dag workflow
- policy violation workflow
- secret leakage workflow
- operator debugging workflow

## Fixture requirements

Each scenario fixture must include:

- `scenario`
- `graph`
- `nodes`
- `focus`
- `expectations`

## Non-negotiable properties

- State-machine conformance is mandatory evidence for battle workflows.
- Node and run transitions must satisfy the state-machine contract and invariant IDs.
- Replay battle scenarios must include mandatory replay proof assertions and semantic diff evidence.

## Ownership metadata

- Scenario ownership and retention metadata live in `evidence/battle/metadata.json`.
- Required fields per scenario:
  - `grade`
  - `why_exists`
  - `delete_review`

## Trust property mapping

- Trust properties and scenario coverage are normative in `configs/policy/battle_trust_properties.json`.
- Every battle scenario must map to one or more trust properties.
- Orphan mappings and orphan metadata entries are rejected by battle drift checks.

## Verification gates

- `cargo nextest run` executes `battle_workflow_harness_contracts`.
- `make test-all` must keep battle checks green.
- `bijux-dev-dag foundation` must include `battle-suite-mandatory` in repo governance checks.

## SOURCE: docs/spec/MODELED_AND_FUTURE_SURFACES.md
# Modeled and future surfaces

## Scope

This document is the single authority for modeled-only and future-only behavior.

## Modeled only (not production execution modes)

- remote coordination behavior used for contract validation
- container backend and batch backend execution models
- federated scheduling and geo coordination simulations

## Future only

- production remote orchestration rollouts
- platform-scale governance productization
- advanced automation beyond current battle-proven trust surfaces

## Documentation rule

Future-only claims are not allowed in normative implementation contracts unless linked to this document and explicitly marked as future.

## Related surfaces

- `docs/spec/CURRENT_IMPLEMENTED_CAPABILITIES.md`
- `docs/reference/SUPPORT_AND_COMPATIBILITY_MATRICES.md`
- `docs/spec/ADOPTION_SURFACES.md`
