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
- 70: Public runtime API mapped and documented in `runtime_public_api_map.md`.
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
- `docs/architecture/runtime_scope_v2.md`
- `docs/reports/foundation/runtime_public_api_map.md`
- `docs/reports/foundation/runtime_boundary_report.md`
- `crates/bijux-dev-dag/tests/runtime_scope_v2_guardrails.rs`
