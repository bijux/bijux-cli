# Runtime Scope and Quarantine Policy

Status: accepted
Owner: runtime maintainers
Date: 2026-03-09

## Decision
Runtime core stays focused on deterministic execution and stable APIs. Experimental and speculative semantics remain quarantined and cannot leak into default operator or identity surfaces.

## Consequences
- Runtime public API remains minimal and durable.
- Quarantined modules require explicit lifecycle and graduation criteria.

## Merged Decision Record
This ADR is standalone. The historical decision text merged into this record is included below.

### SOURCE: 20260309-RUNTIME-CONTRACTION-GOVERNANCE.md
# Runtime Contraction v2

## Decisions 61-80
- 61: Runtime ownership fixed to execution kernel, backend binding, cache, policy, replay, diagnostics.
- 62: Speculative policy/governance modules are marked `move` in `runtime_scope_v2.json`.
- 63: Future-platform runtime content is marked `move` and tracked in runtime boundary report.
- 64: Empty wrapper modules removed from `runtime_core` to eliminate roadmap-only shells.
- 65: Trivial wrapper modules collapsed by removing unused empty files and declarations.
- 66: Adapter surfaces retained; duplicate wrappers disallowed by top-level freeze guard.
- 67: Execution plan home fixed to `runtime_core/planning/execution_plan.rs`.
- 68: Planning bridge home fixed to `runtime_core/planning/planner.rs`.
- 69: Duplicate runtime_core scheduler wrapper surface removed.
- 70: Public runtime API mapped and documented in `RUNTIME_PUBLIC_API_MAP.md`.
- 71: `runtime_core` internal module exports tightened to `pub(crate)` module map only.
- 72: Runtime API map generated in docs reports.
- 73: Scope guardrails test blocks untracked module additions and invalid decisions.
- 74: Runtime dependency and boundary checks remain enforced in `bijux-dev-dag` guardrail tests.
- 75: Runtime formatting/output concerns constrained by guardrails and policy checks.
- 76: Repo-governance logic remains outside runtime and enforced by crate boundary tests.
- 77: Runtime hot-path policy now disallows docs/config dependency creep through boundary guardrails.
- 78: Runtime boundary report generated under docs reports.
- 79: Runtime fake-backend/fake-storage execution proof already covered by runtime execution contracts.
- 80: Contraction gate is now policy-backed and test-enforced before new runtime growth.

## Artifacts
- `configs/policy/runtime_scope_v2.json`
- `configs/policy/runtime_module_freeze.json`
- `docs/reports/foundation/archive/RUNTIME_SCOPE_V2.md`
- `docs/reports/foundation/RUNTIME_PUBLIC_API_MAP.md`
- `docs/reports/foundation/RUNTIME_BOUNDARY_REPORT.md`
- `crates/bijux-dev-dag/tests/runtime_scope_v2_guardrails.rs`

### SOURCE: 20260309-RUNTIME-QUARANTINE-RATIONALE.md
# Runtime Quarantine Rationale

The repository keeps several broad platform surfaces in-tree for contract continuity and evidence history, but these surfaces are quarantined from kernel-stable claims.

## Why Quarantine Instead of Immediate Removal

- preserve existing evidence and compatibility contracts during scope contraction
- prevent accidental breakage of release governance checks that still reference modeled surfaces
- maintain explicit owner mapping for migration into dedicated repositories
- keep kernel/runtime execution boundaries explicit while migration proceeds

### SOURCE: 20260308-ADVANCED-SEMANTICS-RUNTIME-BOUNDARY.md
# ADR: Advanced Semantics Runtime Boundary End-State

## Status
Accepted

## Context
`bijux-dag` accumulated advanced semantics surfaces spanning distributed control-plane, workflow augmentation, and identity-adjacent policy modeling. These surfaces can cause runtime scope drift when presented as default execution behavior.

## Decision
- Keep only `kernel-relevant`, `runtime-relevant`, and `adapter-relevant` advanced semantics in runtime core scope.
- Keep `speculative` families quarantined behind governance classification and non-default UX boundaries.
- Enforce `expire-or-graduate` lifecycle for speculative modules with owner and target date.
- Require generated inventory and gap reports (`no direct tests`, `no user-facing path`, `no fixtures`) from governance policy.

## Consequences
- Runtime core remains deterministic and execution-focused.
- Speculative modeling can continue without leaking into shipped kernel semantics.
- Promotion path is explicit, test-backed, and owner-backed instead of implicit growth.

## References
- `configs/policy/advanced_semantics_governance.json`
- `docs/spec/ADVANCED_SEMANTICS_SCOPE.md`
- `docs/spec/ADVANCED_SEMANTICS_RETAINED_SURFACES.md`
- `docs/spec/ADVANCED_SEMANTICS_QUARANTINED_SURFACES.md`
- `docs/reports/foundation/speculative_surface_budget.md`
