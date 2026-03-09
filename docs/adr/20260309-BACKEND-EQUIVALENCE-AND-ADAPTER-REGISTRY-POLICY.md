# Backend Equivalence and Adapter Registry Policy

Status: accepted
Owner: backend maintainers
Date: 2026-03-09

## Decision
Backend capability declarations and adapter registry semantics are governed by explicit conformance and equivalence contracts.

## Consequences
- Backend claims require linked conformance evidence.
- Adapter identity/version governance remains strict.

## Merged Decision Record
This ADR is standalone. The historical decision text merged into this record is included below.

### SOURCE: 20260308-BACKEND-SEMANTIC-EQUIVALENCE-GUARANTEES.md
# ADR: Backend Semantic Equivalence Guarantees

- Status: accepted
- Date: 2026-03-08

## Context

Backend diversity (local, kubernetes, hpc, remote) is useful only if semantic
behavior remains trustworthy across adapters and replay workflows.

## Decision

Backend semantic equivalence guarantees are:

1. Core execution semantics remain equivalent across supported backends.
2. Replay and diff compatibility across backend-originated runs remains verified.
3. Divergence behavior is explicit, classified, and operator-visible.
4. Capability matrix and support surfaces remain synchronized with generated evidence.

## Enforcement

- Status mapping:
  - `docs/reports/foundation/BACKEND_EQUIVALENCE_281_300_STATUS_REPORT.md`
- Equivalence and diagnostics reports:
  - `docs/reports/foundation/BACKEND_EQUIVALENCE_REPORT.md`
  - `docs/reports/foundation/BACKEND_DIVERGENCE_DIAGNOSTICS_REPORT.md`
  - `docs/reports/foundation/BACKEND_EQUIVALENCE_DASHBOARD.md`
- Verification suite:
  - `configs/suites/backend_equivalence_verification.json`

## Consequences

- Backend behavior is treated as a governed correctness and portability contract.
- New backend capabilities must preserve equivalence guarantees or declare explicit non-equivalence.

### SOURCE: 20260308-RUNTIME-ADAPTER-REGISTRY-END-STATE.md
# ADR: Runtime Adapter and Registry End State

- Date: 2026-03-08
- Status: Accepted

## Context
Runtime adapter and backend capability behavior must stay deterministic and evidence-backed across releases. Drift in adapter registration, capability query output, or contract validation can silently break replay and portability guarantees.

## Decision
- Keep adapter identity and kind registration deterministic and strict.
- Keep backend capability docs generated from command-aligned outputs only.
- Keep claim-to-evidence mapping generated and release-gated.
- Require direct runtime or release-contract test evidence for each shipped adapter/backend surface.

## Consequences
- Adapter/registry behavior remains predictable for operations and replay.
- Backend capability pages cannot diverge from executable surfaces.
- Release checks fail early when shipped adapter coverage or evidence links regress.
