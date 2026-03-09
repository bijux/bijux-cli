# ADR: Replay Planning Guarantees

- Status: accepted
- Date: 2026-03-08

## Context

Replay planning is a core correctness surface. Operators must be able to trust
that replay behavior is stable across imported runs, deterministic for equivalent
graphs, and diagnosable under corruption and partial-state conditions.

## Decision

We define replay planning guarantees as:

1. Replay planning must support imported runs while preserving lineage semantics.
2. Replay plan construction must be deterministic for equivalent graph semantics.
3. Replay planning diagnostics and schema outputs must remain contract-stable.
4. Replay hardening and mismatch corpus coverage must remain enforced in CI.

## Enforcement

- Status and mapping report:
  - `docs/reports/foundation/REPLAY_PLANNING_201_220_STATUS_REPORT.md`
- Complexity and determinism reports:
  - `docs/reports/foundation/REPLAY_PLANNING_COMPLEXITY_REPORT.md`
  - `docs/reports/foundation/REPLAY_PLANNING_DETERMINISM_REPORT.md`
- Consistency dashboard:
  - `docs/reports/foundation/REPLAY_PLAN_CONSISTENCY_DASHBOARD.md`
- Verification suite:
  - `configs/suites/replay_planning_invariants.json`

## Consequences

- Replay planning becomes an explicit operator-facing trust contract.
- Future replay changes must update governed artifacts and keep invariants green.
