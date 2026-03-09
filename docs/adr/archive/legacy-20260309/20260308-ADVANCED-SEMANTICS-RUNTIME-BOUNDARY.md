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
